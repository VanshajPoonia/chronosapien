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
