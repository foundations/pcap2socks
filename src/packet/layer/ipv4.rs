pub use super::{Layer, LayerType, LayerTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{self, Ipv4Flags, Ipv4OptionPacket, Ipv4Packet, MutableIpv4Packet};
use std::clone::Clone;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::net::Ipv4Addr;

/// Represents an IPv4 layer.
#[derive(Clone, Debug)]
pub struct Ipv4 {
    layer: ipv4::Ipv4,
}

impl Ipv4 {
    /// Creates an `Ipv4`.
    pub fn new(identification: u16, t: LayerType, src: Ipv4Addr, dst: Ipv4Addr) -> Option<Ipv4> {
        let next_level_protocol = match t {
            LayerTypes::Tcp => IpNextHeaderProtocols::Tcp,
            LayerTypes::Udp => IpNextHeaderProtocols::Udp,
            _ => return None,
        };
        Some(Ipv4 {
            layer: ipv4::Ipv4 {
                version: 4,
                header_length: 5,
                dscp: 0,
                ecn: 0,
                total_length: 0,
                identification,
                flags: 0,
                fragment_offset: 0,
                ttl: 128,
                next_level_protocol,
                checksum: 0,
                source: src,
                destination: dst,
                options: vec![],
                payload: vec![],
            },
        })
    }

    /// Creates an `Ipv4` represents an IPv4 fragment.
    pub fn new_more_fragment(
        identification: u16,
        t: LayerType,
        fragment_offset: u16,
        src: Ipv4Addr,
        dst: Ipv4Addr,
    ) -> Option<Ipv4> {
        let ipv4 = Ipv4::new(identification, t, src, dst);
        if let Some(mut ipv4) = ipv4 {
            ipv4.layer.flags = Ipv4Flags::MoreFragments;
            ipv4.layer.fragment_offset = fragment_offset;
            return Some(ipv4);
        };

        None
    }

    /// Creates an `Ipv4` represents an IPv4 last fragment.
    pub fn new_last_fragment(
        identification: u16,
        t: LayerType,
        fragment_offset: u16,
        src: Ipv4Addr,
        dst: Ipv4Addr,
    ) -> Option<Ipv4> {
        let ipv4 = Ipv4::new(identification, t, src, dst);
        if let Some(mut ipv4) = ipv4 {
            ipv4.layer.fragment_offset = fragment_offset;
            return Some(ipv4);
        };

        None
    }

    /// Creates an `Ipv4` according to the given `Ipv4`.
    pub fn from(ipv4: ipv4::Ipv4) -> Ipv4 {
        Ipv4 { layer: ipv4 }
    }

    /// Creates an `Ipv4` according to the given IPv4 packet.
    pub fn parse(packet: &Ipv4Packet) -> Ipv4 {
        Ipv4 {
            layer: ipv4::Ipv4 {
                version: packet.get_version(),
                header_length: packet.get_header_length(),
                dscp: packet.get_dscp(),
                ecn: packet.get_ecn(),
                total_length: packet.get_total_length(),
                identification: packet.get_identification(),
                flags: packet.get_flags(),
                fragment_offset: packet.get_fragment_offset(),
                ttl: packet.get_ttl(),
                next_level_protocol: packet.get_next_level_protocol(),
                checksum: packet.get_checksum(),
                source: packet.get_source(),
                destination: packet.get_destination(),
                options: packet.get_options(),
                payload: vec![],
            },
        }
    }

    /// Creates an `Ipv4` without fragmentation according to an `Ipv4`.
    pub fn defrag(ipv4: &Ipv4) -> Ipv4 {
        Ipv4 {
            layer: ipv4::Ipv4 {
                version: ipv4.layer.version,
                header_length: ipv4.layer.header_length,
                dscp: ipv4.layer.dscp,
                ecn: ipv4.layer.ecn,
                total_length: ipv4.layer.total_length,
                identification: ipv4.get_identification(),
                flags: 0,
                fragment_offset: 0,
                ttl: ipv4.layer.ttl,
                next_level_protocol: ipv4.layer.next_level_protocol,
                checksum: 0,
                source: ipv4.get_src(),
                destination: ipv4.get_dst(),
                options: ipv4.layer.options.clone(),
                payload: vec![],
            },
        }
    }

    /// Get the total length of the layer.
    pub fn get_total_length(&self) -> u16 {
        self.layer.total_length
    }

    /// Get the identification of the layer.
    pub fn get_identification(&self) -> u16 {
        self.layer.identification
    }

    /// Returns if more fragments are follows this `Ipv4`.
    pub fn is_more_fragment(&self) -> bool {
        self.layer.flags & Ipv4Flags::MoreFragments != 0
    }

    /// Get the fragment offset of the layer.
    pub fn get_fragment_offset(&self) -> u16 {
        self.layer.fragment_offset
    }

    /// Returns if the `Ipv4` is a IPv4 fragment.
    pub fn is_fragment(&self) -> bool {
        self.is_more_fragment() || self.get_fragment_offset() > 0
    }

    /// Get the source of the layer.
    pub fn get_src(&self) -> Ipv4Addr {
        self.layer.source
    }

    /// Get the destination of the layer.
    pub fn get_dst(&self) -> Ipv4Addr {
        self.layer.destination
    }
}

impl Display for Ipv4 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut fragment = String::new();
        if self.is_fragment() {
            fragment = format!(", Fragment = {}", self.get_fragment_offset() * 8);
        }

        write!(
            f,
            "{}: {} -> {}, Length = {}{}",
            LayerTypes::Ipv4,
            self.layer.source,
            self.layer.destination,
            self.layer.total_length,
            fragment
        )
    }
}

impl Layer for Ipv4 {
    fn get_type(&self) -> LayerType {
        LayerTypes::Ipv4
    }

    fn get_size(&self) -> usize {
        let mut ipv4_size = Ipv4Packet::packet_size(&self.layer);
        let mut ipv4_options_size = 0;
        for option in &self.layer.options {
            ipv4_size -= 1;
            ipv4_options_size += Ipv4OptionPacket::packet_size(option);
        }

        ipv4_size + ipv4_options_size
    }

    fn serialize(&self, buffer: &mut [u8], n: usize) -> io::Result<usize> {
        let mut packet = MutableIpv4Packet::new(buffer)
            .ok_or(io::Error::new(io::ErrorKind::WriteZero, "buffer too small"))?;

        packet.populate(&self.layer);

        // Fix length
        let header_length = self.get_size();
        if header_length / 4 > u8::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::Other, "IPv4 too big"));
        }
        packet.set_header_length((header_length / 4) as u8);
        if n > u16::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "length too big",
            ));
        }
        packet.set_total_length(n as u16);

        // Compute checksum
        let checksum = ipv4::checksum(&packet.to_immutable());
        packet.set_checksum(checksum);

        Ok(header_length)
    }

    fn serialize_with_payload(&self, buffer: &mut [u8], _: &[u8], n: usize) -> io::Result<usize> {
        self.serialize(buffer, n)
    }
}
