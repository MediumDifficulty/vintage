use anyhow::Result;
use std::{
    fmt::Debug,
    io::{Cursor, Read},
};

use byteorder::{BigEndian, ReadBytesExt};
use evenio::world::World;
use tracing::warn;

use crate::event::PlayerJoinEvent;

use super::{Byte, FByte, FShort, PacketString, SByte, Short};

pub struct PacketReader {
    buffer: Cursor<Vec<u8>>,
}

impl PacketReader {
    pub fn new(data: Vec<u8>) -> PacketReader {
        PacketReader {
            buffer: Cursor::new(data),
        }
    }

    pub fn read_byte(&mut self) -> Result<Byte> {
        Ok(self.buffer.read_u8()?)
    }

    pub fn read_sbyte(&mut self) -> Result<SByte> {
        Ok(self.buffer.read_i8()?)
    }

    pub fn read_fbyte(&mut self) -> Result<FByte> {
        Ok(FByte(self.read_sbyte()?))
    }

    pub fn read_short(&mut self) -> Result<Short> {
        Ok(self.buffer.read_i16::<BigEndian>()?)
    }

    pub fn read_fshort(&mut self) -> Result<FShort> {
        Ok(FShort(self.buffer.read_i16::<BigEndian>()?))
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
    fn deserialise(reader: &mut PacketReader) -> Result<Self>
    where
        Self: Sized;
}

/// Sent by a player joining a server with relevant information. The protocol version is 0x07, unless you're using a client below 0.28.
#[derive(Debug)]
pub struct PlayerIdentPacket {
    protocol_version: Byte,
    username: PacketString,
    verification_key: PacketString,
}

impl C2SPacket for PlayerIdentPacket {
    fn exec(&self, world: &mut World) {
        if self.protocol_version < 7 {
            warn!("Client's protocol version in less than 7")
        }

        world.send(PlayerJoinEvent {
            username: self.username.to_string(),
        })
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self>
    where
        Self: Sized,
    {
        let protocol_version = reader.read_byte()?;
        let username = reader.read_string()?;
        let verification_key = reader.read_string()?;

        Ok(Self {
            protocol_version,
            username,
            verification_key,
        })
    }
}

/// Sent when a user changes a block. The mode field indicates whether a block was created (0x01) or destroyed (0x00).
///
/// Block type is always the type player is holding, even when destroying.
///
/// Client assumes that this command packet always succeeds, and so draws the new block immediately. To disallow block creation, server must send back Set Block packet with the old block type.
#[derive(Debug)]
pub struct SetBlockPacket {
    x: Short,
    y: Short,
    z: Short,
    mode: Byte,
    block_type: Byte,
}

impl C2SPacket for SetBlockPacket {
    fn exec(&self, world: &mut World) {
        todo!()
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self>
    where
        Self: Sized,
    {
        let x = reader.read_short()?;
        let y = reader.read_short()?;
        let z = reader.read_short()?;
        let mode = reader.read_byte()?;
        let block_type = reader.read_byte()?;

        Ok(Self {
            x,
            y,
            z,
            mode,
            block_type,
        })
    }
}

/// Sent frequently (even while not moving) by the player with the player's current location and orientation. Player ID is always -1 (255), referring to itself.
#[derive(Debug)]
pub struct PositionPacket {
    player_id: SByte,
    x: FShort,
    y: FShort,
    z: FShort,
    yaw: Byte,
    pitch: Byte,
}

impl C2SPacket for PositionPacket {
    fn exec(&self, world: &mut World) {
        todo!()
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self>
    where
        Self: Sized,
    {
        let player_id = reader.read_sbyte()?;
        let x = reader.read_fshort()?;
        let y = reader.read_fshort()?;
        let z = reader.read_fshort()?;
        let yaw = reader.read_byte()?;
        let pitch = reader.read_byte()?;

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

/// Contain chat messages sent by player. Player ID is always -1 (255), referring to itself.
#[derive(Debug)]
pub struct MessagePacket {
    player_id: SByte,
    message: PacketString,
}

impl C2SPacket for MessagePacket {
    fn exec(&self, world: &mut World) {
        todo!()
    }

    fn deserialise(reader: &mut PacketReader) -> Result<Self>
    where
        Self: Sized,
    {
        let player_id = reader.read_sbyte()?;
        let message = reader.read_string()?;

        Ok(Self { player_id, message })
    }
}
