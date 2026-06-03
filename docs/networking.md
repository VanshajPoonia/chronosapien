# ChronoOS Networking

Status: partially implemented, partially verified in QEMU.

ChronoOS contains code for QEMU's RTL8139 PCI network card. The first network
milestone is intentionally small:

- static IPv4 address: `10.0.2.15`
- QEMU user-mode gateway: `10.0.2.2`
- netmask: `255.255.255.0`
- ARP and UDP only
- polling receive path
- compact observability counters for TX/RX, ARP, UDP, malformed RX, last event,
  and last error
- no DHCP, TCP, DNS, sockets, packet capture, or real-hardware support yet

2026-06-02 QEMU evidence:

- `qemu-system-x86_64` booted single-core with RTL8139 attached.
- Serial log `/private/tmp/chronoos-net-20260602-162000.serial.log` reached
  `[CHRONO] boot complete`.
- Serial log showed `net: rtl8139 found bus=0 device=3 function=0 io=0xc000`
  and MAC `52:54:00:12:34:56`.
- ARP and UDP behavior were not verified. `hostfwd=udp::9000-:9000` conflicted
  with the host UDP listener, and later QEMU monitor key injection submitted
  `n7et` and `neett` instead of clean `net` commands.
- Host UDP log `/private/tmp/chronoos-net-20260602-162000.host-udp.log` remained
  0 bytes.

Run scripts add an RTL8139 device with a fixed MAC:

```powershell
-netdev "user,id=net0,hostfwd=udp::9000-:9000"
-device "rtl8139,netdev=net0,mac=52:54:00:12:34:56"
```

Inside the shell:

```text
CHRONO/84> net status
ChronoOS network status
NIC: rtl8139 initialized
MAC: 52:54:00:12:34:56
IP:  10.0.2.15 (static)
GW:  10.0.2.2 unresolved
MASK: 255.255.255.0
TX:  0 packets
RX:  0 packets
ARP: 0 requests sent, 0 replies received
UDP: 0 sent, 0 received
Malformed RX: 0
Last event: rtl8139 initialized
Last error: none
```

Use `net config` to show the static QEMU setup, `net arp` to ask for the
gateway MAC, and `net send` to send a default UDP packet to `10.0.2.2:9000`
during an intentional verification run.

## Shell Observability Commands

- `net` / `net status`: current NIC, MAC, static IP, gateway, counters, last
  event, last error, and conservative verification status.
- `net config`: static address, gateway, netmask, default UDP payload, and
  unsupported protocol boundary.
- `net arp`: educational ARP explanation plus the existing gateway ARP request.
- `net udp`: educational UDP explanation and send syntax without transmitting.
- `net send [ip port text]`: transmit the existing default or custom UDP payload
  after warning that ARP/UDP needs runtime verification.
- `net log`: compact counters and last event/error only; not packet capture.
- `net demo`: read-only walkthrough that routes the operator through status,
  config, ARP, UDP, and intentional send commands.
- `net roadmap`: future DHCP, DNS, TCP, sockets, and broader hardware work.

These counters are real code-path counters, but they are small diagnostics, not a
full network monitor. If a stat is zero, it means that path has not been observed
since the driver initialized in the current boot.

## Ethernet Frames

Ethernet is the link-layer packet format the NIC sends and receives. ChronoOS
uses Ethernet II frames:

```text
destination MAC  6 bytes
source MAC       6 bytes
EtherType        2 bytes
payload          46-1500 bytes
CRC              4 bytes, generated/checked by hardware
```

The destination MAC says which machine on the local link should receive the
frame. The source MAC is our NIC address. EtherType tells the receiver how to
interpret the payload:

- `0x0806` means ARP
- `0x0800` means IPv4

The Ethernet CRC exists on the wire, but the RTL8139 handles it for normal
driver code, so ChronoOS does not build it manually.

## ARP

IPv4 uses addresses like `10.0.2.2`, but Ethernet needs MAC addresses like
`52:54:00:12:34:56`. ARP maps IPv4 addresses to MAC addresses on the local
network.

An ARP request says:

```text
Who has 10.0.2.2? Tell 10.0.2.15.
```

The Ethernet destination for an ARP request is broadcast:

```text
ff:ff:ff:ff:ff:ff
```

An ARP reply says:

```text
10.0.2.2 is at aa:bb:cc:dd:ee:ff.
```

ChronoOS stores that gateway MAC and logs:

```text
[CHRONO] net: ARP reply from 10.0.2.2
```

## IPv4 Headers

ChronoOS builds a minimal 20-byte IPv4 header:

```text
version/IHL        1 byte   version 4, header length 5 words
DSCP/ECN           1 byte   zero
total length       2 bytes  IP header + UDP header + payload
identification     2 bytes  small fixed value for now
flags/fragment     2 bytes  no fragmentation
TTL                1 byte   64
protocol           1 byte   17 for UDP
header checksum    2 bytes  one's-complement checksum
source IP          4 bytes  10.0.2.15
destination IP     4 bytes  target host
```

The checksum covers only the IPv4 header. It is computed by summing 16-bit words
with carry wraparound and then bitwise-inverting the result.

## UDP Headers

UDP is the smallest useful transport layer for this milestone:

```text
source port       2 bytes
destination port  2 bytes
length            2 bytes  UDP header + payload
checksum          2 bytes
payload           N bytes
```

For IPv4, a UDP checksum of zero means "checksum not used." ChronoOS uses zero
for this first stack so the implementation can focus on moving packets.

Incoming UDP packets addressed to `10.0.2.15` are logged to serial:

```text
[CHRONO] net: UDP rx 12 bytes from 10.0.2.2:9000 -> 9000: hello
```

## Testing With QEMU

Start a UDP listener on the host:

```powershell
ncat -ul 9000
```

On macOS without `ncat`, `/usr/bin/nc -ul 9000` is enough for a guest-to-host
listener. Do not run the listener on the same host port as a QEMU `hostfwd`
binding unless you have confirmed the bind order works on that host.

Boot ChronoOS with `.\scripts\run.ps1` or `.\scripts\run-custom.ps1`, then run:

```text
net status
net config
net arp
net log
net send
net log
```

The host listener should receive the default ChronoOS payload after the gateway
MAC has been learned.

To send from host to guest, use the run-script UDP forward on host port `9000`.
The shell polls the RTL8139 receive ring once per timer tick, so receive logs
appear during normal shell idle time.

Keep DHCP, DNS, TCP, sockets, packet capture, and broad hardware networking as
roadmap/design-only until ARP/UDP has clean packet evidence.
