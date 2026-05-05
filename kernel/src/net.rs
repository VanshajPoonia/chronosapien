//! Polling-first RTL8139 Ethernet, ARP, IPv4, and UDP support.

use core::cell::UnsafeCell;
use core::cmp::min;

use crate::{io, pci, println};

const RTL8139_VENDOR_ID: u16 = 0x10EC;
const RTL8139_DEVICE_ID: u16 = 0x8139;

const IDR0: u16 = 0x00;
const TX_STATUS0: u16 = 0x10;
const TX_ADDR0: u16 = 0x20;
const RX_BUF: u16 = 0x30;
const COMMAND: u16 = 0x37;
const RX_BUF_PTR: u16 = 0x38;
const INTERRUPT_MASK: u16 = 0x3C;
const INTERRUPT_STATUS: u16 = 0x3E;
const TX_CONFIG: u16 = 0x40;
const RX_CONFIG: u16 = 0x44;
const CONFIG1: u16 = 0x52;

const COMMAND_RESET: u8 = 1 << 4;
const COMMAND_RX_ENABLE: u8 = 1 << 3;
const COMMAND_TX_ENABLE: u8 = 1 << 2;
const COMMAND_RX_BUFFER_EMPTY: u8 = 1 << 0;

const RX_OK: u16 = 1 << 0;
const RX_BUFFER_LEN: usize = 8192 + 16 + 1500;
const TX_BUFFER_LEN: usize = 2048;
const TX_BUFFER_COUNT: usize = 4;

const ETHERNET_HEADER_LEN: usize = 14;
const IPV4_HEADER_LEN: usize = 20;
const UDP_HEADER_LEN: usize = 8;
const ARP_PACKET_LEN: usize = 28;
const ETHER_TYPE_ARP: u16 = 0x0806;
const ETHER_TYPE_IPV4: u16 = 0x0800;
const IP_PROTOCOL_UDP: u8 = 17;

const LOCAL_IP: [u8; 4] = [10, 0, 2, 15];
const GATEWAY_IP: [u8; 4] = [10, 0, 2, 2];
const NETMASK: [u8; 4] = [255, 255, 255, 0];
const BROADCAST_MAC: [u8; 6] = [0xFF; 6];
const DEFAULT_UDP_PORT: u16 = 9000;
const DEFAULT_UDP_PAYLOAD: &[u8] = b"hello from ChronoOS";

#[derive(Clone, Copy)]
struct Rtl8139 {
    io_base: u16,
    mac: [u8; 6],
    tx_index: usize,
    rx_offset: usize,
}

#[derive(Clone, Copy)]
struct NetState {
    initialized: bool,
    nic: Option<Rtl8139>,
    gateway_mac: Option<[u8; 6]>,
    tx_packets: u64,
    rx_packets: u64,
}

impl NetState {
    const fn new() -> Self {
        Self {
            initialized: false,
            nic: None,
            gateway_mac: None,
            tx_packets: 0,
            rx_packets: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub initialized: bool,
    pub mac: Option<[u8; 6]>,
    pub ip: [u8; 4],
    pub netmask: [u8; 4],
    pub gateway_ip: [u8; 4],
    pub gateway_mac: Option<[u8; 6]>,
    pub tx_packets: u64,
    pub rx_packets: u64,
}

#[repr(align(4))]
struct RxBuffer {
    bytes: [u8; RX_BUFFER_LEN],
}

#[repr(align(4))]
struct TxBuffers {
    bytes: [[u8; TX_BUFFER_LEN]; TX_BUFFER_COUNT],
}

struct GlobalNet(UnsafeCell<NetState>);
struct GlobalRx(UnsafeCell<RxBuffer>);
struct GlobalTx(UnsafeCell<TxBuffers>);

unsafe impl Sync for GlobalNet {}
unsafe impl Sync for GlobalRx {}
unsafe impl Sync for GlobalTx {}

static NET: GlobalNet = GlobalNet(UnsafeCell::new(NetState::new()));
static RX_BUFFER: GlobalRx = GlobalRx(UnsafeCell::new(RxBuffer {
    bytes: [0; RX_BUFFER_LEN],
}));
static TX_BUFFERS: GlobalTx = GlobalTx(UnsafeCell::new(TxBuffers {
    bytes: [[0; TX_BUFFER_LEN]; TX_BUFFER_COUNT],
}));

pub fn init() {
    let Some(device) = pci::find_device(RTL8139_VENDOR_ID, RTL8139_DEVICE_ID) else {
        crate::serial_println!("[CHRONO] net: rtl8139 not found");
        return;
    };

    let bar0 = device.bar0();
    if bar0 & 1 == 0 {
        crate::serial_println!("[CHRONO] net: rtl8139 BAR0 is not an I/O BAR");
        return;
    }

    let io_base = (bar0 & !3) as u16;
    device.enable_io_and_bus_mastering();

    crate::serial_println!(
        "[CHRONO] net: rtl8139 found bus={} device={} function={} io={:#x}",
        device.bus,
        device.device,
        device.function,
        io_base
    );

    let mut nic = Rtl8139 {
        io_base,
        mac: [0; 6],
        tx_index: 0,
        rx_offset: 0,
    };

    unsafe {
        io::outb(io_base + CONFIG1, 0x00);
        io::outb(io_base + COMMAND, COMMAND_RESET);
        for _ in 0..100_000 {
            if io::inb(io_base + COMMAND) & COMMAND_RESET == 0 {
                break;
            }
        }

        for index in 0..nic.mac.len() {
            nic.mac[index] = io::inb(io_base + IDR0 + index as u16);
        }

        io::outl(io_base + RX_BUF, rx_buffer_address());
        io::outw(io_base + INTERRUPT_MASK, 0);
        io::outw(io_base + INTERRUPT_STATUS, 0xFFFF);
        io::outl(io_base + RX_CONFIG, 0x0000_008F);
        io::outl(io_base + TX_CONFIG, 0);
        io::outb(io_base + COMMAND, COMMAND_RX_ENABLE | COMMAND_TX_ENABLE);
        io::outw(io_base + RX_BUF_PTR, 0);
    }

    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let state = &mut *NET.0.get();
        state.initialized = true;
        state.nic = Some(nic);
        state.gateway_mac = None;
        state.tx_packets = 0;
        state.rx_packets = 0;
    });

    crate::serial_println!(
        "[CHRONO] net: mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        nic.mac[0],
        nic.mac[1],
        nic.mac[2],
        nic.mac[3],
        nic.mac[4],
        nic.mac[5]
    );
}

pub fn poll() {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let state = &mut *NET.0.get();
        let Some(mut nic) = state.nic else {
            return;
        };

        poll_rtl8139(state, &mut nic);
        state.nic = Some(nic);
    });
}

pub fn run(command: &str) -> bool {
    let command = command.trim();
    if command != "net" && !command.starts_with("net ") {
        return false;
    }

    let rest = command.strip_prefix("net").unwrap_or("").trim_start();
    if rest.is_empty() {
        print_status();
        return true;
    }

    if rest == "arp" {
        send_gateway_arp();
        return true;
    }

    if rest == "send" {
        send_udp(GATEWAY_IP, DEFAULT_UDP_PORT, DEFAULT_UDP_PAYLOAD);
        return true;
    }

    if let Some(args) = rest.strip_prefix("send ") {
        run_send_command(args);
        return true;
    }

    println!("Usage: net | net arp | net send [ip port text]");
    true
}

fn run_send_command(args: &str) {
    let args = args.trim_start();
    let Some((ip_text, rest)) = split_token(args) else {
        println!("Usage: net send <ip> <port> <text>");
        return;
    };
    let Some((port_text, payload)) = split_token(rest.trim_start()) else {
        println!("Usage: net send <ip> <port> <text>");
        return;
    };

    let Some(ip) = parse_ipv4(ip_text) else {
        println!("invalid IP address");
        return;
    };
    let Some(port) = parse_u16(port_text) else {
        println!("invalid UDP port");
        return;
    };

    send_udp(ip, port, payload.trim_start().as_bytes());
}

fn print_status() {
    let snapshot = snapshot();

    match snapshot.mac {
        Some(mac) => println!(
            "MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        ),
        None => println!("MAC: not initialized"),
    }
    println!(
        "IP:  {}.{}.{}.{} (static)",
        snapshot.ip[0], snapshot.ip[1], snapshot.ip[2], snapshot.ip[3]
    );
    println!(
        "GW:  {}.{}.{}.{} {}",
        snapshot.gateway_ip[0],
        snapshot.gateway_ip[1],
        snapshot.gateway_ip[2],
        snapshot.gateway_ip[3],
        if snapshot.gateway_mac.is_some() {
            "learned"
        } else {
            "unresolved"
        }
    );
    println!(
        "MASK: {}.{}.{}.{}",
        snapshot.netmask[0], snapshot.netmask[1], snapshot.netmask[2], snapshot.netmask[3]
    );
    println!("TX:  {} packets", snapshot.tx_packets);
    println!("RX:  {} packets", snapshot.rx_packets);
}

fn snapshot() -> Snapshot {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let state = &*NET.0.get();

        Snapshot {
            initialized: state.initialized,
            mac: state.nic.map(|nic| nic.mac),
            ip: LOCAL_IP,
            netmask: NETMASK,
            gateway_ip: GATEWAY_IP,
            gateway_mac: state.gateway_mac,
            tx_packets: state.tx_packets,
            rx_packets: state.rx_packets,
        }
    })
}

fn send_gateway_arp() {
    let mut frame = [0u8; 64];
    let Some((mac, _)) = local_link_info() else {
        println!("net: rtl8139 not initialized");
        return;
    };

    build_ethernet_header(&mut frame, BROADCAST_MAC, mac, ETHER_TYPE_ARP);
    build_arp_packet(
        &mut frame[ETHERNET_HEADER_LEN..ETHERNET_HEADER_LEN + ARP_PACKET_LEN],
        1,
        mac,
        LOCAL_IP,
        [0; 6],
        GATEWAY_IP,
    );
    transmit(&frame[..ETHERNET_HEADER_LEN + ARP_PACKET_LEN]);
    crate::serial_println!(
        "[CHRONO] net: ARP request for {}.{}.{}.{}",
        GATEWAY_IP[0],
        GATEWAY_IP[1],
        GATEWAY_IP[2],
        GATEWAY_IP[3]
    );
}

fn send_udp(dest_ip: [u8; 4], dest_port: u16, payload: &[u8]) {
    let Some((mac, gateway_mac)) = local_link_info() else {
        println!("net: rtl8139 not initialized");
        return;
    };

    let Some(dest_mac) = gateway_mac else {
        println!("net: gateway unresolved; sending ARP first");
        send_gateway_arp();
        return;
    };

    let payload_len = min(payload.len(), 512);
    let udp_len = UDP_HEADER_LEN + payload_len;
    let ip_len = IPV4_HEADER_LEN + udp_len;
    let frame_len = ETHERNET_HEADER_LEN + ip_len;
    let mut frame = [0u8; 1514];

    build_ethernet_header(&mut frame, dest_mac, mac, ETHER_TYPE_IPV4);
    build_ipv4_header(
        &mut frame[ETHERNET_HEADER_LEN..ETHERNET_HEADER_LEN + IPV4_HEADER_LEN],
        ip_len as u16,
        LOCAL_IP,
        dest_ip,
        IP_PROTOCOL_UDP,
    );
    build_udp_header(
        &mut frame[ETHERNET_HEADER_LEN + IPV4_HEADER_LEN..],
        DEFAULT_UDP_PORT,
        dest_port,
        udp_len as u16,
    );
    frame[ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + UDP_HEADER_LEN..frame_len]
        .copy_from_slice(&payload[..payload_len]);

    transmit(&frame[..frame_len]);
    crate::serial_println!(
        "[CHRONO] net: UDP tx {} bytes to {}.{}.{}.{}:{}",
        payload_len,
        dest_ip[0],
        dest_ip[1],
        dest_ip[2],
        dest_ip[3],
        dest_port
    );
}

fn local_link_info() -> Option<([u8; 6], Option<[u8; 6]>)> {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let state = &*NET.0.get();
        state.nic.map(|nic| (nic.mac, state.gateway_mac))
    })
}

fn transmit(frame: &[u8]) {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let state = &mut *NET.0.get();
        let Some(mut nic) = state.nic else {
            return;
        };

        let index = nic.tx_index % TX_BUFFER_COUNT;
        let tx_buffers = &mut *TX_BUFFERS.0.get();
        let length = min(frame.len(), TX_BUFFER_LEN);
        tx_buffers.bytes[index][..length].copy_from_slice(&frame[..length]);

        let tx_address = tx_buffers.bytes[index].as_ptr() as u32;
        io::outl(nic.io_base + TX_ADDR0 + (index as u16 * 4), tx_address);
        io::outl(nic.io_base + TX_STATUS0 + (index as u16 * 4), length as u32);

        nic.tx_index = (nic.tx_index + 1) % TX_BUFFER_COUNT;
        state.nic = Some(nic);
        state.tx_packets = state.tx_packets.saturating_add(1);
    });
}

unsafe fn poll_rtl8139(state: &mut NetState, nic: &mut Rtl8139) {
    let rx_buffer = &mut (*RX_BUFFER.0.get()).bytes;
    let mut processed = 0;

    while io::inb(nic.io_base + COMMAND) & COMMAND_RX_BUFFER_EMPTY == 0 && processed < 4 {
        if nic.rx_offset + 4 >= RX_BUFFER_LEN {
            nic.rx_offset = 0;
        }

        let status = read_le_u16(rx_buffer, nic.rx_offset);
        let length = read_le_u16(rx_buffer, nic.rx_offset + 2) as usize;

        if status & RX_OK == 0 || length < 4 || length > 1600 {
            nic.rx_offset = 0;
            io::outw(nic.io_base + RX_BUF_PTR, 0);
            break;
        }

        let packet_len = length - 4;
        let packet_start = nic.rx_offset + 4;
        let packet_end = packet_start + packet_len;

        if packet_end <= RX_BUFFER_LEN {
            handle_rx_packet(state, &rx_buffer[packet_start..packet_end]);
            state.rx_packets = state.rx_packets.saturating_add(1);
        }

        nic.rx_offset = (nic.rx_offset + length + 4 + 3) & !3;
        if nic.rx_offset >= 8192 {
            nic.rx_offset -= 8192;
        }

        io::outw(
            nic.io_base + RX_BUF_PTR,
            (nic.rx_offset as u16).wrapping_sub(16),
        );
        io::outw(nic.io_base + INTERRUPT_STATUS, 0xFFFF);
        processed += 1;
    }
}

fn handle_rx_packet(state: &mut NetState, packet: &[u8]) {
    if packet.len() < ETHERNET_HEADER_LEN {
        return;
    }

    let ether_type = be_u16(packet, 12);
    match ether_type {
        ETHER_TYPE_ARP => handle_arp(state, packet),
        ETHER_TYPE_IPV4 => handle_ipv4(packet),
        _ => {}
    }
}

fn handle_arp(state: &mut NetState, packet: &[u8]) {
    if packet.len() < ETHERNET_HEADER_LEN + ARP_PACKET_LEN {
        return;
    }

    let arp = &packet[ETHERNET_HEADER_LEN..];
    let opcode = be_u16(arp, 6);
    let sender_mac = [arp[8], arp[9], arp[10], arp[11], arp[12], arp[13]];
    let sender_ip = [arp[14], arp[15], arp[16], arp[17]];
    let target_ip = [arp[24], arp[25], arp[26], arp[27]];

    if opcode == 2 && sender_ip == GATEWAY_IP && target_ip == LOCAL_IP {
        state.gateway_mac = Some(sender_mac);
        crate::serial_println!(
            "[CHRONO] net: ARP reply from {}.{}.{}.{}",
            sender_ip[0],
            sender_ip[1],
            sender_ip[2],
            sender_ip[3]
        );
    }
}

fn handle_ipv4(packet: &[u8]) {
    if packet.len() < ETHERNET_HEADER_LEN + IPV4_HEADER_LEN {
        return;
    }

    let ip = &packet[ETHERNET_HEADER_LEN..];
    let header_len = ((ip[0] & 0x0F) as usize) * 4;
    if header_len < IPV4_HEADER_LEN || ip.len() < header_len + UDP_HEADER_LEN {
        return;
    }

    let total_len = be_u16(ip, 2) as usize;
    if total_len > ip.len() || ip[9] != IP_PROTOCOL_UDP {
        return;
    }

    let src_ip = [ip[12], ip[13], ip[14], ip[15]];
    let dest_ip = [ip[16], ip[17], ip[18], ip[19]];
    if dest_ip != LOCAL_IP {
        return;
    }

    let udp = &ip[header_len..total_len];
    if udp.len() < UDP_HEADER_LEN {
        return;
    }

    let src_port = be_u16(udp, 0);
    let dest_port = be_u16(udp, 2);
    let udp_len = be_u16(udp, 4) as usize;
    if udp_len < UDP_HEADER_LEN || udp_len > udp.len() {
        return;
    }

    let payload = &udp[UDP_HEADER_LEN..udp_len];
    crate::serial_print!(
        "[CHRONO] net: UDP rx {} bytes from {}.{}.{}.{}:{} -> {}: ",
        payload.len(),
        src_ip[0],
        src_ip[1],
        src_ip[2],
        src_ip[3],
        src_port,
        dest_port
    );
    for byte in payload.iter().copied().take(48) {
        let printable = if byte.is_ascii_graphic() || byte == b' ' {
            byte
        } else {
            b'.'
        };
        crate::serial_print!("{}", printable as char);
    }
    crate::serial_println!("");
}

fn build_ethernet_header(frame: &mut [u8], dest: [u8; 6], source: [u8; 6], ether_type: u16) {
    frame[0..6].copy_from_slice(&dest);
    frame[6..12].copy_from_slice(&source);
    put_be_u16(frame, 12, ether_type);
}

fn build_arp_packet(
    packet: &mut [u8],
    opcode: u16,
    sender_mac: [u8; 6],
    sender_ip: [u8; 4],
    target_mac: [u8; 6],
    target_ip: [u8; 4],
) {
    put_be_u16(packet, 0, 1);
    put_be_u16(packet, 2, ETHER_TYPE_IPV4);
    packet[4] = 6;
    packet[5] = 4;
    put_be_u16(packet, 6, opcode);
    packet[8..14].copy_from_slice(&sender_mac);
    packet[14..18].copy_from_slice(&sender_ip);
    packet[18..24].copy_from_slice(&target_mac);
    packet[24..28].copy_from_slice(&target_ip);
}

fn build_ipv4_header(
    header: &mut [u8],
    total_len: u16,
    source_ip: [u8; 4],
    dest_ip: [u8; 4],
    protocol: u8,
) {
    header[..IPV4_HEADER_LEN].fill(0);
    header[0] = 0x45;
    header[1] = 0;
    put_be_u16(header, 2, total_len);
    put_be_u16(header, 4, 1);
    put_be_u16(header, 6, 0);
    header[8] = 64;
    header[9] = protocol;
    header[12..16].copy_from_slice(&source_ip);
    header[16..20].copy_from_slice(&dest_ip);
    let checksum = ipv4_checksum(&header[..IPV4_HEADER_LEN]);
    put_be_u16(header, 10, checksum);
}

fn build_udp_header(header: &mut [u8], source_port: u16, dest_port: u16, udp_len: u16) {
    put_be_u16(header, 0, source_port);
    put_be_u16(header, 2, dest_port);
    put_be_u16(header, 4, udp_len);
    put_be_u16(header, 6, 0);
}

fn ipv4_checksum(header: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut index = 0;

    while index + 1 < header.len() {
        sum += be_u16(header, index) as u32;
        index += 2;
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}

fn rx_buffer_address() -> u32 {
    unsafe { (*RX_BUFFER.0.get()).bytes.as_mut_ptr() as u32 }
}

fn read_le_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn be_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn put_be_u16(bytes: &mut [u8], offset: usize, value: u16) {
    let [high, low] = value.to_be_bytes();
    bytes[offset] = high;
    bytes[offset + 1] = low;
}

