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
