//! Packet implementation for creating [`Packet`]s along with types
//! used by the router for creating and decoding contents / responses
//!
//! Also contains the decoding and encoding logic for tokio codec
//! [`PacketCodec`]

use bitflags::bitflags;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{fmt::Debug, hash::Hash, sync::Arc};
use std::{io, ops::Deref};
use tdf::{
    serialize_vec, DecodeResult, TdfDeserialize, TdfDeserializer, TdfSerialize, TdfSerializer,
    TdfStringifier,
};
use tokio_util::codec::{Decoder, Encoder};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PacketFlags: u8 {
        const FLAG_DEFAULT = 0;
        const FLAG_RESPONSE = 32;
        const FLAG_NOTIFY = 64;
        const FLAG_KEEP_ALIVE = 128;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PacketHeader {
    pub component: u16,
    pub command: u16,
    pub seq: u32,
    pub flags: PacketFlags,
    pub notify: u8,
    pub unused: u8,
}

impl PacketHeader {
    pub const fn notify(component: u16, command: u16) -> Self {
        Self {
            component,
            command,
            seq: 0,
            flags: PacketFlags::FLAG_NOTIFY,
            notify: 0,
            unused: 0,
        }
    }

    pub const fn request(component: u16, command: u16, seq: u32) -> Self {
        Self {
            component,
            command,
            seq: 0,
            flags: PacketFlags::FLAG_DEFAULT,
            notify: 0,
            unused: 0,
        }
    }

    pub const fn response(&self) -> Self {
        let mut header = *self;
        header.flags = header.flags.union(PacketFlags::FLAG_RESPONSE);
        header
    }

    /// Checks if the component and command of this packet header matches
    /// that of the other packet header
    ///
    /// `other` The packet header to compare to
    pub fn path_matches(&self, other: &PacketHeader) -> bool {
        self.component.eq(&other.component) && self.command.eq(&other.command)
    }
}

/// Structure for Blaze packets contains the contents of the packet
/// and the header for identification.
///
/// Packets can be cloned with little memory usage increase because
/// the content is stored as Bytes.
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub pre_msg: Bytes,
    pub body: Bytes,
}

impl Packet {
    /// Creates a packet from its raw components
    ///
    /// `header`   The packet header
    /// `contents` The encoded packet contents
    pub fn raw(header: PacketHeader, contents: Vec<u8>) -> Self {
        Self {
            header,
            pre_msg: Bytes::new(),
            body: Bytes::from(contents),
        }
    }

    /// Creates a packet from its raw components
    /// where the contents are empty
    ///
    /// `header` The packet header
    pub const fn raw_empty(header: PacketHeader) -> Self {
        Self {
            header,
            pre_msg: Bytes::new(),
            body: Bytes::new(),
        }
    }

    pub fn response<C: TdfSerialize>(packet: &Packet, contents: C) -> Self {
        Self {
            header: packet.header.response(),
            pre_msg: Bytes::new(),
            body: Bytes::from(serialize_vec(&contents)),
        }
    }

    pub fn respond<C: TdfSerialize>(&self, contents: C) -> Self {
        Self::response(self, contents)
    }

    pub fn response_raw(packet: &Packet, contents: Vec<u8>) -> Self {
        Self {
            header: packet.header.response(),
            pre_msg: Bytes::new(),
            body: Bytes::from(contents),
        }
    }

    pub const fn response_empty(packet: &Packet) -> Self {
        Self {
            header: packet.header.response(),
            pre_msg: Bytes::new(),
            body: Bytes::new(),
        }
    }

    pub const fn respond_empty(&self) -> Self {
        Self::response_empty(self)
    }

    pub fn notify<C: TdfSerialize>(component: u16, command: u16, contents: C) -> Packet {
        Self {
            header: PacketHeader::notify(component, command),
            pre_msg: Bytes::new(),
            body: Bytes::from(serialize_vec(&contents)),
        }
    }

    pub fn notify_raw(component: u16, command: u16, contents: Vec<u8>) -> Packet {
        Self {
            header: PacketHeader::notify(component, command),
            pre_msg: Bytes::new(),
            body: Bytes::from(contents),
        }
    }

    pub fn notify_empty(component: u16, command: u16) -> Packet {
        Self {
            header: PacketHeader::notify(component, command),
            pre_msg: Bytes::new(),
            body: Bytes::new(),
        }
    }

    pub fn request<C: TdfSerialize>(component: u16, command: u16, seq: u32, contents: C) -> Packet {
        Self {
            header: PacketHeader::request(component, command, seq),
            pre_msg: Bytes::new(),
            body: Bytes::from(serialize_vec(&contents)),
        }
    }

    pub fn request_raw(component: u16, command: u16, seq: u32, contents: Vec<u8>) -> Packet {
        Self {
            header: PacketHeader::request(component, command, seq),
            pre_msg: Bytes::new(),
            body: Bytes::from(contents),
        }
    }

    pub fn request_empty(component: u16, command: u16, seq: u32) -> Packet {
        Self {
            header: PacketHeader::request(component, command, seq),
            pre_msg: Bytes::new(),
            body: Bytes::new(),
        }
    }

    /// Attempts to decode the contents bytes of this packet into the
    /// provided Codec type value.
    pub fn decode<'de, C: TdfDeserialize<'de>>(&'de self) -> DecodeResult<C> {
        let mut reader = TdfDeserializer::new(&self.body);
        C::deserialize(&mut reader)
    }

    /// Attempts to read a packet from the provided
    /// bytes source
    ///
    /// `src` The bytes to read from
    pub fn read(src: &mut BytesMut) -> Option<Self> {
        if src.len() < 16 {
            return None;
        }

        let length = src.get_u32();
        let pre_length = src.get_u16();
        let component = src.get_u16();
        let command = src.get_u16();
        let mut seq = [0u8; 4];
        src.take(3).copy_to_slice(&mut seq[1..]);
        let seq = u32::from_be_bytes(seq);
        let mty = src.get_u8();
        let flags = PacketFlags::from_bits_retain(mty);
        let notify = src.get_u8();
        let unused = src.get_u8();

        if src.len() < pre_length as usize + length as usize {
            return None;
        }
        let pre_msg = src.split_to(pre_length as usize);
        let body = src.split_to(length as usize);

        Some(Packet {
            header: PacketHeader {
                component,
                command,
                seq,
                flags,
                notify,
                unused,
            },
            pre_msg: pre_msg.freeze(),
            body: body.freeze(),
        })
    }

    /// Writes the contents and header of the packet
    /// onto the dst source of bytes
    ///
    /// `dst` The destination buffer
    pub fn write(&self, dst: &mut BytesMut) {
        dst.put_u32(self.body.len() as u32);
        dst.put_u16(self.pre_msg.len() as u16);
        dst.put_u16(self.header.component);
        dst.put_u16(self.header.command);

        let seq = self.header.seq.to_be_bytes();
        dst.put_slice(&seq[1..]);

        let ty = self.header.flags.bits();
        dst.put_u8(ty);
        dst.put_u8(self.header.notify);
        dst.put_u8(self.header.unused);
        dst.extend_from_slice(&self.pre_msg);
        dst.extend_from_slice(&self.body);
    }
}

/// Tokio codec for encoding and decoding packets
pub struct PacketCodec;

/// Decoder implementation
impl Decoder for PacketCodec {
    type Error = io::Error;
    type Item = Packet;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut read_src = src.clone();
        let result = Packet::read(&mut read_src);

        if result.is_some() {
            *src = read_src;
        }

        Ok(result)
    }
}

/// Encoder implementation for owned packets
impl Encoder<Packet> for PacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.write(dst);
        Ok(())
    }
}

/// Encoder implementation for borrowed packets
impl Encoder<&Packet> for PacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: &Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.write(dst);
        Ok(())
    }
}

/// Encoder implementation for arc reference packets
impl Encoder<Arc<Packet>> for PacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Arc<Packet>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.write(dst);
        Ok(())
    }
}

/// Structure wrapping a from request type to include a packet
/// header to allow the response type to be created
pub struct Request<T: FromRequest> {
    /// The decoded request type
    pub req: T,
    /// The packet header from the request
    pub header: PacketHeader,
}

/// Deref implementation so that the request fields can be
/// directly accessed
impl<T: FromRequest> Deref for Request<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

impl<T: FromRequest> Request<T> {
    /// Creates a response from the provided response type value
    /// returning a Response structure which can be used as a Route
    /// repsonse
    ///
    /// `res` The into response type implementation
    pub fn response<E>(&self, res: E) -> Response
    where
        E: TdfSerialize,
    {
        Response(Packet {
            header: self.header.response(),
            pre_msg: Bytes::new(),
            body: Bytes::from(serialize_vec(&res)),
        })
    }
}

/// Wrapping structure for raw Bytes structures that can
/// be used as packet response
pub struct PacketBody(Bytes);

impl<T> From<T> for PacketBody
where
    T: TdfSerialize,
{
    fn from(value: T) -> Self {
        let bytes = serialize_vec(&value);
        let bytes = Bytes::from(bytes);
        PacketBody(bytes)
    }
}

/// Type for route responses that have already been turned into
/// packets usually for lifetime reasons
pub struct Response(Packet);

impl IntoResponse for Response {
    /// Simply provide the already compute response
    fn into_response(self, _req: &Packet) -> Packet {
        self.0
    }
}

impl IntoResponse for PacketBody {
    fn into_response(self, req: &Packet) -> Packet {
        Packet {
            header: req.header.response(),
            pre_msg: Bytes::new(),
            body: self.0,
        }
    }
}

impl<T: FromRequest> FromRequest for Request<T> {
    fn from_request(req: &Packet) -> DecodeResult<Self> {
        let inner = T::from_request(req)?;
        let header = req.header;
        Ok(Self { req: inner, header })
    }
}

/// Trait implementing by structures which can be created from a request
/// packet and is used for the arguments on routing functions
pub trait FromRequest: Sized + Send + 'static {
    /// Takes the value from the request returning a decode result of
    /// whether the value could be created
    ///
    /// `req` The request packet
    fn from_request(req: &Packet) -> DecodeResult<Self>;
}

impl<D> FromRequest for D
where
    for<'de> D: TdfDeserialize<'de> + Send + 'static,
{
    fn from_request(req: &Packet) -> DecodeResult<Self> {
        req.decode()
    }
}

/// Trait for a type that can be converted into a packet
/// response using the header from the request packet
pub trait IntoResponse: 'static {
    /// Into packet conversion
    fn into_response(self, req: &Packet) -> Packet;
}

/// Into response imeplementation for encodable responses
/// which just calls res.respond
impl<E> IntoResponse for E
where
    E: TdfSerialize + 'static,
{
    fn into_response(self, req: &Packet) -> Packet {
        req.respond(self)
    }
}

/// Wrapper over a packet structure to provde debug logging
/// with names resolved for the component
pub struct PacketDebug<'a> {
    /// Reference to the packet itself
    pub packet: &'a Packet,

    /// Decide whether to display the contents of the packet
    pub minified: bool,
}

impl<'a> Debug for PacketDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Append basic header information
        let header = &self.packet.header;

        writeln!(f, "Component: {:#06x}", header.component)?;
        writeln!(f, "Command: {:#06x}", header.command)?;

        writeln!(f, "Flags: {:?}", header.flags)?;
        writeln!(f, "Seq: {}", &header.seq)?;
        writeln!(f, "Notif: {}", &header.notify)?;
        writeln!(f, "Unused: {}", &header.unused)?;

        // Skip remaining if the message shouldn't contain its content
        if self.minified {
            return Ok(());
        }

        if !self.packet.pre_msg.is_empty() {
            let mut r = TdfDeserializer::new(&self.packet.pre_msg);
            let mut out = String::new();

            out.push_str("{\n");
            let mut s = TdfStringifier::new(r, &mut out);

            let _ = s.stringify();

            if out.len() == 2 {
                // Remove new line if nothing else was appended
                out.pop();
            }

            out.push('}');

            writeln!(f, "Pre Message: {}", out)?;
        }

        let mut r = TdfDeserializer::new(&self.packet.body);
        let mut out = String::new();

        out.push_str("{\n");
        let mut s = TdfStringifier::new(r, &mut out);

        let _ = s.stringify();

        if out.len() == 2 {
            // Remove new line if nothing else was appended
            out.pop();
        }

        out.push('}');

        write!(f, "Content: {}", out)
    }
}
