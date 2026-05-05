# ChronoOS Networking

ChronoOS currently targets QEMU's RTL8139 PCI network card. The first network
milestone is intentionally small:

- static IPv4 address: `10.0.2.15`
- QEMU user-mode gateway: `10.0.2.2`
- netmask: `255.255.255.0`
- ARP and UDP only
- polling receive path
- no DHCP, TCP, DNS, or real-hardware support yet

Run scripts add an RTL8139 device with a fixed MAC:

```powershell
-netdev "user,id=net0,hostfwd=udp::9000-:9000"
-device "rtl8139,netdev=net0,mac=52:54:00:12:34:56"
```

Inside the shell:

```text
CHRONO/84> net
MAC: 52:54:00:12:34:56
IP:  10.0.2.15 (static)
GW:  10.0.2.2 unresolved
MASK: 255.255.255.0
TX:  0 packets
RX:  0 packets
```

Use `net arp` to ask for the gateway MAC and `net send` to send a default UDP
packet to `10.0.2.2:9000`.

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

Boot ChronoOS with `.\scripts\run.ps1` or `.\scripts\run-custom.ps1`, then run:

```text
net arp
net send
```

The host listener should receive the default ChronoOS payload after the gateway
MAC has been learned.

To send from host to guest, use the run-script UDP forward on host port `9000`.
The shell polls the RTL8139 receive ring once per timer tick, so receive logs
appear during normal shell idle time.
