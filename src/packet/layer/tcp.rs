pub use super::{Layer, LayerType, LayerTypes};
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags, TcpOptionPacket, TcpPacket};
use std::clone::Clone;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::net::Ipv4Addr;

/// Represents a TCP packet.
#[derive(Clone, Debug)]
pub struct Tcp {
    pub layer: tcp::Tcp,
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
}

impl Tcp {
    /// Creates a `Tcp` represents a TCP ACK.
    pub fn new_ack(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
        window: u16,
    ) -> Tcp {
        Tcp {
            layer: tcp::Tcp {
                source: src,
                destination: dst,
                sequence,
                acknowledgement,
                data_offset: 5,
                reserved: 0,
                flags: TcpFlags::ACK,
                window,
                checksum: 0,
                urgent_ptr: 0,
                options: vec![],
                payload: vec![],
            },
            src: src_ip_addr,
            dst: dst_ip_addr,
        }
    }

    /// Creates a `Tcp` represents a TCP ACK/SYN.
    pub fn new_ack_syn(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
        window: u16,
    ) -> Tcp {
        let mut tcp = Tcp::new_ack(
            src_ip_addr,
            dst_ip_addr,
            src,
            dst,
            sequence,
            acknowledgement,
            window,
        );
        tcp.layer.flags |= TcpFlags::SYN;
        tcp
    }

    /// Creates a `Tcp` represents a TCP ACK/RST.
    pub fn new_ack_rst(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
        window: u16,
    ) -> Tcp {
        let mut tcp = Tcp::new_rst(
            src_ip_addr,
            dst_ip_addr,
            src,
            dst,
            sequence,
            acknowledgement,
            window,
        );
        tcp.layer.flags |= TcpFlags::ACK;
        tcp
    }

    /// Creates a `Tcp` represents a TCP ACK/FIN.
    pub fn new_ack_fin(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
        window: u16,
    ) -> Tcp {
        let mut tcp = Tcp::new_ack(
            src_ip_addr,
            dst_ip_addr,
            src,
            dst,
            sequence,
            acknowledgement,
            window,
        );
        tcp.layer.flags |= TcpFlags::FIN;
        tcp
    }

    /// Creates a `Tcp` represents a TCP RST.
    pub fn new_rst(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
        window: u16,
    ) -> Tcp {
        let mut tcp = Tcp::new_ack(
            src_ip_addr,
            dst_ip_addr,
            src,
            dst,
            sequence,
            acknowledgement,
            window,
        );
        tcp.layer.flags = TcpFlags::RST;
        tcp
    }

    /// Creates a `Tcp` according to the given `Tcp`.
    pub fn from(tcp: tcp::Tcp, src: Ipv4Addr, dst: Ipv4Addr) -> Tcp {
        Tcp {
            layer: tcp,
            src,
            dst,
        }
    }

    /// Creates a `Tcp` according to the given TCP packet, source and destination.
    pub fn parse(packet: &TcpPacket, src: Ipv4Addr, dst: Ipv4Addr) -> Tcp {
        Tcp {
            layer: tcp::Tcp {
                source: packet.get_source(),
                destination: packet.get_destination(),
                sequence: packet.get_sequence(),
                acknowledgement: packet.get_acknowledgement(),
                data_offset: packet.get_data_offset(),
                reserved: packet.get_reserved(),
                flags: packet.get_flags(),
                window: packet.get_window(),
                checksum: packet.get_checksum(),
                urgent_ptr: packet.get_urgent_ptr(),
                options: packet.get_options(),
                payload: vec![],
            },
            src,
            dst,
        }
    }

    /// Get the source IP address of the layer.
    pub fn get_src_ip_addr(&self) -> Ipv4Addr {
        self.src
    }

    /// Get the destination IP address of the layer.
    pub fn get_dst_ip_addr(&self) -> Ipv4Addr {
        self.dst
    }

    /// Get the source of the layer.
    pub fn get_src(&self) -> u16 {
        self.layer.source
    }

    /// Get the destination of the layer.
    pub fn get_dst(&self) -> u16 {
        self.layer.destination
    }

    /// Get the sequence of the layer.
    pub fn get_sequence(&self) -> u32 {
        self.layer.sequence
    }

    /// Get the acknowledgement of the layer.
    pub fn get_acknowledgement(&self) -> u32 {
        self.layer.acknowledgement
    }

    /// Get the string represents the flags of the layer.
    pub fn get_flag_string(&self) -> String {
        let mut flags = String::from("[");
        if self.is_syn() {
            flags = flags + "S";
        }
        if self.is_rst() {
            flags = flags + "R";
        }
        if self.is_fin() {
            flags = flags + "F";
        }
        if self.is_ack() {
            flags = flags + ".";
        }
        flags = flags + "]";

        flags
    }

    /// Get the window size of the layer.
    pub fn get_window(&self) -> u16 {
        self.layer.window
    }

    /// Returns if the `Tcp` is a TCP acknowledgement.
    pub fn is_ack(&self) -> bool {
        self.layer.flags & TcpFlags::ACK != 0
    }

    /// Returns if the `Tcp` is a TCP acknowledgement and finish.
    pub fn is_ack_fin(&self) -> bool {
        self.is_ack() && self.is_fin()
    }

    /// Returns if the `Tcp` is a TCP reset.
    pub fn is_rst(&self) -> bool {
        self.layer.flags & TcpFlags::RST != 0
    }

    /// Returns if the `Tcp` is a TCP synchronization.
    pub fn is_syn(&self) -> bool {
        self.layer.flags & TcpFlags::SYN != 0
    }

    /// Returns if the `Tcp` is a TCP finish.
    pub fn is_fin(&self) -> bool {
        self.layer.flags & TcpFlags::FIN != 0
    }

    /// Returns if the `Tcp` is a TCP reset or finish.
    pub fn is_rst_or_fin(&self) -> bool {
        self.is_rst() || self.is_fin()
    }

    // Returns if the `Tcp` has zero window.
    pub fn is_zero_window(&self) -> bool {
        self.layer.window == 0
    }
}

impl Display for Tcp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} {}",
            LayerTypes::Tcp,
            self.layer.source,
            self.layer.destination,
            self.get_flag_string()
        )
    }
}

impl Layer for Tcp {
    fn get_type(&self) -> LayerType {
        LayerTypes::Tcp
    }

    fn get_size(&self) -> usize {
        let mut tcp_size = TcpPacket::packet_size(&self.layer);
        let mut tcp_options_size = 0;
        for option in &self.layer.options {
            tcp_size -= 1;
            tcp_options_size += TcpOptionPacket::packet_size(option);
        }

        tcp_size + tcp_options_size
    }

    fn serialize(&self, buffer: &mut [u8], _: usize) -> io::Result<usize> {
        let mut packet = MutableTcpPacket::new(buffer)
            .ok_or(io::Error::new(io::ErrorKind::WriteZero, "buffer too small"))?;

        packet.populate(&self.layer);

        // Fix length
        let header_length = self.get_size();
        if header_length / 4 > u8::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::Other, "TCP too big"));
        }
        packet.set_data_offset((header_length / 4) as u8);

        // Compute checksum
        let checksum = tcp::ipv4_checksum(
            &packet.to_immutable(),
            &self.get_src_ip_addr(),
            &self.get_dst_ip_addr(),
        );
        packet.set_checksum(checksum);

        Ok(header_length)
    }

    fn serialize_with_payload(
        &self,
        buffer: &mut [u8],
        payload: &[u8],
        n: usize,
    ) -> io::Result<usize> {
        let mut packet = MutableTcpPacket::new(buffer)
            .ok_or(io::Error::new(io::ErrorKind::WriteZero, "buffer too small"))?;

        packet.populate(&self.layer);

        // Copies payload
        packet.set_payload(payload);

        // Fix length
        let header_length = self.get_size();
        if header_length / 4 > u8::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::Other, "TCP too big"));
        }
        packet.set_data_offset((header_length / 4) as u8);

        // Compute checksum
        let checksum = tcp::ipv4_checksum(
            &packet.to_immutable(),
            &self.get_src_ip_addr(),
            &self.get_dst_ip_addr(),
        );
        packet.set_checksum(checksum);

        Ok(header_length + n)
    }
}
