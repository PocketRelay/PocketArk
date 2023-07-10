use std::{io, sync::Arc};

use bitflags::bitflags;
use blaze_pk::codec::Encodable;
use bytes::{Buf, BufMut, BytesMut};
use hyper::body::Bytes;
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
    pub fn response(&self) -> Self {
        Self {
            component: self.component,
            command: self.command,
            seq: self.seq,
            flags: self.flags | PacketFlags::FLAG_RESPONSE,
            notify: 0,
            unused: 0,
        }
    }

    pub fn notify(component: u16, command: u16) -> Self {
        Self {
            component,
            command,
            seq: 0,
            flags: PacketFlags::FLAG_NOTIFY,
            notify: 1,
            unused: 0,
        }
    }
}

#[derive(Debug)]
pub struct Packet {
    pub header: PacketHeader,
    pre_msg: Bytes,
    pub body: Bytes,
}

impl Packet {
    pub fn respond<C: Encodable>(&self, contents: C) -> Self {
        Self {
            header: self.header.response(),
            pre_msg: Bytes::new(),
            body: Bytes::from(contents.encode_bytes()),
        }
    }

    pub fn notify<C: Encodable>(component: u16, command: u16, contents: C) -> Self {
        Self {
            header: PacketHeader::notify(component, command),
            pre_msg: Bytes::new(),
            body: Bytes::from(contents.encode_bytes()),
        }
    }

    pub fn respond_empty(&self) -> Self {
        Self {
            header: self.header.response(),
            pre_msg: Bytes::new(),
            body: Bytes::new(),
        }
    }
}

impl Packet {
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

    pub fn read(src: &mut BytesMut) -> Option<Packet> {
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
