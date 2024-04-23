use anyhow::Result;
use core::fmt::Debug;
use std::str::FromStr;

use self::c2s::{C2SPacket, C2SPacketEntry, PacketReader};

pub mod c2s;
pub mod extension;
pub mod listener;
pub mod s2c;
pub mod util;

pub type Byte = u8;
pub type SByte = i8;
pub type Short = i16;
pub type ByteArray = [u8; 1024];

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Default, Debug)]
pub struct ClientPacketRegistry {
    packets: Vec<Option<ClientPacketRegistryEntry>>,
}

#[derive(Debug, Clone)]
pub struct ClientPacketRegistryEntry {
    size: usize,
    deserialiser: fn(&mut PacketReader) -> Result<Box<dyn C2SPacket>>,
}

impl ClientPacketRegistry {
    pub fn register<P: C2SPacketEntry>(&mut self) {
        let id = P::ID;

        if self.packets.len() <= id as usize {
            self.packets.resize(id as usize + 1, None);
        }

        self.packets[id as usize] = Some(ClientPacketRegistryEntry {
            size: P::SIZE,
            deserialiser: P::deserialise,
        });
    }

    pub fn get(&self, id: Byte) -> Option<&ClientPacketRegistryEntry> {
        self.packets[id as usize].as_ref()
    }
}

impl ClientPacketRegistryEntry {
    pub fn deserialise(&self, reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        (self.deserialiser)(reader)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
