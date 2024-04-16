use crate::packet::c2s::PlayerIdent;
use core::fmt::Debug;
use std::str::FromStr;
use anyhow::Result;

use self::c2s::{C2SPacket, Message, PacketReader, Position, SetBlock};

pub mod c2s;
pub mod s2c;

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
        Ok(PacketString(format!("{data: <32}").as_bytes().try_into()?))
    }
}

impl ToString for PacketString {
    fn to_string(&self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap().trim_end_matches(' ').to_string()
    }
}

impl Debug for PacketString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketString").field("data", &self.to_string()).finish()
    }
}

enum_from_primitive! {
#[derive(Debug)]
pub enum ClientPacketID {
    PlayerIdent = 0x00,
    SetBlock    = 0x05,
    Position    = 0x08,
    Message     = 0x0d,
}
}

// pub enum ServerPacketID {
//     PlayerIdent        = 0x00,
//     Ping               = 0x01,
//     LevelInit          = 0x02,
//     LevelDataChunk     = 0x03,
//     LevelFinalise      = 0x04,
//     SetBlock           = 0x06,
//     SpawnPlayer        = 0x07,
//     PlayerTeleport     = 0x08,
//     PlayerPosOriUpdate = 0x09,
//     PlayerPosUpdate    = 0x0a,
//     PlayerOriUpdate    = 0x0b,
//     DespawnPlayer      = 0x0c,
//     Message            = 0x0d,
//     DisconnectPlayer   = 0x0e,
//     UpdateUserType     = 0x0f,
// }

impl ClientPacketID {
    pub fn size(&self) -> usize {
        match self {
            ClientPacketID::PlayerIdent => 1 + 2 * PacketString::LENGTH + 1,
            ClientPacketID::SetBlock => 5,
            ClientPacketID::Position => 8,
            ClientPacketID::Message => 2 + 1024,
        }
    }

    pub fn deserialise(&self, reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        match self {
            ClientPacketID::PlayerIdent => Ok(Box::new(PlayerIdent::deserialise(reader)?)),
            ClientPacketID::SetBlock => Ok(Box::new(SetBlock::deserialise(reader)?)),
            ClientPacketID::Position => Ok(Box::new(Position::deserialise(reader)?)),
            ClientPacketID::Message => Ok(Box::new(Message::deserialise(reader)?)),
        }
    }
}