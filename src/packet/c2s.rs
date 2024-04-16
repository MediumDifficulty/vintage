use std::{fmt::Debug, io::{Cursor, Read}};
use anyhow::Result;

use byteorder::{BigEndian, ReadBytesExt};
use evenio::world::World;
use tracing::warn;

use crate::event::PlayerJoinEvent;

use super::{Byte, FByte, FShort, PacketString, SByte, Short};

pub struct PacketReader {
    buffer: Cursor<Vec<u8>>
}

impl PacketReader {
    pub fn new(data: Vec<u8>) -> PacketReader {
        PacketReader {
            buffer: Cursor::new(data)
        }
    }

    pub fn read_byte(&mut self) -> Byte {
        self.buffer.read_u8().unwrap()
    }

    pub fn read_sbyte(&mut self) -> SByte {
        self.buffer.read_i8().unwrap()
    }

    pub fn read_fbyte(&mut self) -> FByte {
        let b = self.read_sbyte();
        FByte(b)
    }

    pub fn read_short(&mut self) -> Short {
        self.buffer.read_i16::<BigEndian>().unwrap()
    }

    pub fn read_fshort(&mut self) -> FShort {
        let s = self.buffer.read_i16::<BigEndian>().unwrap();
        FShort(s)
    }

    pub fn read_string(&mut self) -> Result<PacketString> {
        let mut buf = [0; PacketString::LENGTH];

        self.buffer.read_exact(&mut buf)?;

        Ok(PacketString::new(buf))
    }

    pub fn read_byte_array(&mut self) -> Result<[u8; 1024]> {
        let mut buf = [0; 1024];

        self.buffer.read_exact(&mut buf)?;

        Ok(buf)
    }
}

pub trait C2SPacket: Send + Sync + Debug {
    fn exec(&self, world: &mut World);
    fn deserialise(reader: &mut PacketReader) -> Result<Self> where Self: Sized;
}


#[derive(Debug)]
pub struct PlayerIdent {
    protocol_version: Byte,
    username: PacketString,
    verification_key: PacketString,
}

impl C2SPacket for PlayerIdent {
    fn exec(&self, world: &mut World) {
        if self.protocol_version < 7 {
            warn!("Client's protocol version in less than 7")
        }

        world.send(PlayerJoinEvent {
            username: self.username.to_string(),
        })
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self> where Self: Sized {
        let protocol_version = reader.read_byte();
        let username = reader.read_string()?;
        let verification_key = reader.read_string()?;

        Ok(Self {
            protocol_version,
            username,
            verification_key,
        })
    }
}

#[derive(Debug)]
pub struct SetBlock {
    x: Short,
    y: Short,
    z: Short,
    mode: Byte,
    block_type: Byte,
}

impl C2SPacket for SetBlock {
    fn exec(&self, world: &mut World) {
        todo!()
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self> where Self: Sized {
        let x = reader.read_short();
        let y = reader.read_short();
        let z = reader.read_short();
        let mode = reader.read_byte();
        let block_type = reader.read_byte();

        Ok(Self {
            x,
            y,
            z,
            mode,
            block_type,
        })
    }
}

#[derive(Debug)]
pub struct Position {
    player_id: SByte,
    x: FShort,
    y: FShort,
    z: FShort,
    yaw: Byte,
    pitch: Byte,
}

impl C2SPacket for Position {
    fn exec(&self, world: &mut World) {
        todo!()
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self> where Self: Sized {
        let player_id = reader.read_sbyte();
        let x = reader.read_fshort();
        let y = reader.read_fshort();
        let z = reader.read_fshort();
        let yaw = reader.read_byte();
        let pitch = reader.read_byte();

        Ok(Self {
            player_id,
            x,
            y,
            z,
            yaw,
            pitch,
        })
    }
}

#[derive(Debug)]
pub struct Message {
    player_id: SByte,
    message: PacketString,
}

impl C2SPacket for Message {
    fn exec(&self, world: &mut World) {
        todo!()
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self> where Self: Sized {
        let player_id = reader.read_sbyte();
        let message = reader.read_string()?;

        Ok(Self {
            player_id,
            message,
        })
    }
}