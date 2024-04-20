use crate::networking::c2s::PlayerIdentPacket;
use anyhow::Result;
use core::fmt::Debug;
use std::str::FromStr;

use self::c2s::{C2SPacket, MessagePacket, PacketReader, PositionPacket, SetBlockPacket};

pub mod c2s;
pub mod listener;
pub mod s2c;
pub mod util;

pub type Byte = u8;
pub type SByte = i8;
pub type Short = i16;
pub type ByteArray = [u8; 1024];

#[derive(Debug)]
pub struct FByte(pub i8);

impl From<FByte> for f32 {
    fn from(fb: FByte) -> Self {
        fb.0 as f32 / 2f32.powf(5.)
    }
}

impl From<f32> for FByte {
    fn from(value: f32) -> Self {
        FByte((value * 2f32.powf(5.)) as i8)
    }
}

#[derive(Debug)]
pub struct FShort(pub i16);

impl From<FShort> for f32 {
    fn from(fs: FShort) -> Self {
        fs.0 as f32 / 2f32.powf(5.)
    }
}

impl From<f32> for FShort {
    fn from(value: f32) -> Self {
        FShort((value * 2f32.powf(5.)) as i16)
    }
}

pub struct PacketString(pub [u8; Self::LENGTH]);

impl PacketString {
    pub fn new(data: [u8; Self::LENGTH]) -> PacketString {
        PacketString(data)
    }

    pub const LENGTH: usize = 64;
}

impl FromStr for PacketString {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<PacketString> {
        Ok(PacketString(format!("{data: <64}").as_bytes().try_into()?))
    }
}

impl ToString for PacketString {
    fn to_string(&self) -> String {
        String::from_utf8(self.0.to_vec())
            .unwrap()
            .trim_end_matches(' ')
            .to_string()
    }
}

impl Debug for PacketString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketString")
            .field("data", &self.to_string())
            .finish()
    }
}

enum_from_primitive! {
#[derive(Debug, PartialEq, Eq)]
pub enum ClientPacketID {
    PlayerIdent = 0x00,
    SetBlock    = 0x05,
    Position    = 0x08,
    Message     = 0x0d,
}
}

impl ClientPacketID {
    pub fn size(&self) -> usize {
        match self {
            ClientPacketID::PlayerIdent => 1 + 2 * PacketString::LENGTH + 1,
            ClientPacketID::SetBlock => 3 * 2 + 2,
            ClientPacketID::Position => 1 + 3 * 2 + 2,
            ClientPacketID::Message => 1 + PacketString::LENGTH,
        }
    }

    pub fn deserialise(&self, reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        match self {
            ClientPacketID::PlayerIdent => Ok(Box::new(PlayerIdentPacket::deserialise(reader)?)),
            ClientPacketID::SetBlock => Ok(Box::new(SetBlockPacket::deserialise(reader)?)),
            ClientPacketID::Position => Ok(Box::new(PositionPacket::deserialise(reader)?)),
            ClientPacketID::Message => Ok(Box::new(MessagePacket::deserialise(reader)?)),
        }
    }
}
