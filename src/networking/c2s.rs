use anyhow::{Context, Result};
use enum_primitive::FromPrimitive;
use glam::{uvec3, vec3};
use std::{
    fmt::Debug,
    io::{Cursor, Read},
};

use byteorder::{BigEndian, ReadBytesExt};
use evenio::world::World;
use tracing::warn;

use crate::{
    event::{PlayerJoinEvent, PlayerMessageEvent, PlayerMoveEvent, SetBlockEvent},
    world::{self, ClientConnection, Rotation},
};

use super::{
    listener::ClientInfo, util::angle_to_f32, Byte, FByte, FShort, PacketString, SByte, Short,
};

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

    pub fn read_packet_string(&mut self) -> Result<PacketString> {
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
    fn exec(&self, world: &mut World, client_info: &ClientInfo) -> Result<()>;
}

pub trait C2SPacketEntry {
    const ID: Byte;
    const SIZE: usize;

    fn deserialise(reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>>;
}

/// Sent by a player joining a server with relevant information. The protocol version is 0x07, unless you're using a client below 0.28.
#[derive(Debug)]
pub struct PlayerIdentPacket {
    protocol_version: Byte,
    username: PacketString,
    #[allow(unused)]
    verification_key: PacketString,
    padding: Byte,
}

impl C2SPacketEntry for PlayerIdentPacket {
    const ID: Byte = 0x00;
    const SIZE: usize = 1 + 2 * PacketString::LENGTH + 1;

    fn deserialise(reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        let protocol_version = reader.read_byte()?;
        let username = reader.read_packet_string()?;
        let verification_key = reader.read_packet_string()?;
        let padding = reader.read_byte()?;

        Ok(Box::new(Self {
            protocol_version,
            username,
            verification_key,
            padding,
        }))
    }
}

impl C2SPacket for PlayerIdentPacket {
    fn exec(&self, world: &mut World, client_info: &ClientInfo) -> Result<()> {
        if self.protocol_version < 7 {
            warn!("Client's protocol version in less than 7")
        }

        let player = world.spawn();

        *client_info.player_id.lock().unwrap() = Some(player);

        world.insert(
            player,
            ClientConnection {
                sender: client_info.packet_sender.clone(),
                addr: client_info.addr,
            },
        );

        world.send(PlayerJoinEvent {
            entity_id: player,
            username: self.username.to_string(),
            cpe: self.padding == 0x42,
        });

        Ok(())
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

impl C2SPacketEntry for SetBlockPacket {
    const ID: Byte = 0x05;
    const SIZE: usize = 3 * 2 + 2;

    fn deserialise(reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        let x = reader.read_short()?;
        let y = reader.read_short()?;
        let z = reader.read_short()?;
        let mode = reader.read_byte()?;
        let block_type = reader.read_byte()?;

        Ok(Box::new(Self {
            x,
            y,
            z,
            mode,
            block_type,
        }))
    }
}

impl C2SPacket for SetBlockPacket {
    fn exec(&self, world: &mut World, _client_info: &ClientInfo) -> Result<()> {
        world.send(SetBlockEvent {
            block: world::Block::from_u8(self.block_type).context("Invalid block id")?,
            placed: self.mode == 1,
            pos: uvec3(self.x as u32, self.y as u32, self.z as u32),
        });

        Ok(())
    }
}

/// Sent frequently (even while not moving) by the player with the player's current location and orientation. Player ID is always -1 (255), referring to itself.
#[derive(Debug)]
pub struct PositionPacket {
    #[allow(unused)]
    player_id: SByte,
    x: FShort,
    y: FShort,
    z: FShort,
    yaw: Byte,
    pitch: Byte,
}

impl C2SPacketEntry for PositionPacket {
    const ID: Byte = 0x08;
    const SIZE: usize = 1 + 3 * 2 + 2;

    fn deserialise(reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        let player_id = reader.read_sbyte()?;
        let x = reader.read_fshort()?;
        let y = reader.read_fshort()?;
        let z = reader.read_fshort()?;
        let yaw = reader.read_byte()?;
        let pitch = reader.read_byte()?;

        Ok(Box::new(Self {
            player_id,
            x,
            y,
            z,
            yaw,
            pitch,
        }))
    }
}

impl C2SPacket for PositionPacket {
    fn exec(&self, world: &mut World, client_info: &ClientInfo) -> Result<()> {
        let entity_id = client_info.player_id.lock().unwrap();

        world.send(PlayerMoveEvent {
            pos: vec3(self.x.into(), self.y.into(), self.z.into()),
            rot: Rotation {
                pitch: angle_to_f32(self.pitch),
                yaw: angle_to_f32(self.yaw),
            },
            entity_id: entity_id.unwrap(),
        });

        Ok(())
    }
}

/// Contain chat messages sent by player. Player ID is always -1 (255), referring to itself.
#[derive(Debug)]
pub struct MessagePacket {
    #[allow(unused)]
    player_id: SByte,
    message: PacketString,
}

impl C2SPacketEntry for MessagePacket {
    const ID: Byte = 0x0d;
    const SIZE: usize = 1 + PacketString::LENGTH;

    fn deserialise(reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        let player_id = reader.read_sbyte()?;
        let message = reader.read_packet_string()?;

        Ok(Box::new(Self { player_id, message }))
    }
}

impl C2SPacket for MessagePacket {
    fn exec(&self, world: &mut World, client_info: &ClientInfo) -> Result<()> {
        world.send(PlayerMessageEvent {
            entity_id: (*client_info.player_id.lock().unwrap()).unwrap(),
            message: self.message.to_string(),
        });

        Ok(())
    }
}
