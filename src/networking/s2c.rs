pub mod util;

use std::{
    fmt::Debug,
    io::{Cursor, Write},
    str::FromStr,
};

use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};

use super::{extension::Int, Byte, ByteArray, FByte, FShort, PacketString, SByte, Short};

pub struct PacketWriter {
    buffer: Cursor<Vec<u8>>,
}

impl PacketWriter {
    pub fn new(data: Vec<u8>) -> PacketWriter {
        PacketWriter {
            buffer: Cursor::new(data),
        }
    }

    pub fn new_empty() -> PacketWriter {
        PacketWriter {
            buffer: Cursor::new(Vec::new()),
        }
    }

    pub fn new_with_capacity(capacity: usize) -> PacketWriter {
        PacketWriter {
            buffer: Cursor::new(Vec::with_capacity(capacity)),
        }
    }

    pub fn write_packet(&mut self, packet: &dyn S2CPacket) -> Result<()> {
        self.write_byte(packet.id())?;
        packet.serialise(self)?;
        Ok(())
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buffer.into_inner()
    }

    pub fn write_byte(&mut self, b: Byte) -> Result<()> {
        Ok(self.buffer.write_u8(b)?)
    }

    pub fn write_sbyte(&mut self, b: SByte) -> Result<()> {
        Ok(self.buffer.write_i8(b)?)
    }

    pub fn write_fbyte(&mut self, b: &FByte) -> Result<()> {
        Ok(self.buffer.write_i8(b.0)?)
    }

    pub fn write_short(&mut self, s: Short) -> Result<()> {
        Ok(self.buffer.write_i16::<BigEndian>(s)?)
    }

    pub fn write_fshort(&mut self, s: &FShort) -> Result<()> {
        Ok(self.buffer.write_i16::<BigEndian>(s.0)?)
    }

    pub fn write_string(&mut self, s: &str) -> Result<()> {
        Ok(self.buffer.write_all(&PacketString::from_str(s)?.0)?)
    }

    pub fn write_packet_string(&mut self, s: &PacketString) -> Result<()> {
        Ok(self.buffer.write_all(&s.0)?)
    }

    pub fn write_byte_array(&mut self, buf: &[u8; 1024]) -> Result<()> {
        Ok(self.buffer.write_all(buf)?)
    }

    pub fn write_int(&mut self, i: Int) -> Result<()> {
        Ok(self.buffer.write_i32::<BigEndian>(i)?)
    }
}

pub trait S2CPacket: Send + Sync + Debug {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()>;
    fn id(&self) -> Byte;
}

#[derive(Debug)]
pub struct ServerIdentPacket {
    pub protocol_version: Byte,
    pub server_name: PacketString,
    pub server_motd: PacketString,
    pub user_type: Byte,
}

impl S2CPacket for ServerIdentPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_byte(self.protocol_version)?;
        writer.write_packet_string(&self.server_name)?;
        writer.write_packet_string(&self.server_motd)?;
        writer.write_byte(self.user_type)
    }

    fn id(&self) -> Byte {
        0x00
    }
}

#[derive(Debug)]
pub struct PingPacket;

impl S2CPacket for PingPacket {
    fn serialise(&self, _writer: &mut PacketWriter) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> Byte {
        0x01
    }
}

#[derive(Debug)]
pub struct LevelInitPacket;

impl S2CPacket for LevelInitPacket {
    fn serialise(&self, _writer: &mut PacketWriter) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> Byte {
        0x02
    }
}

#[derive(Debug)]
pub struct LevelDataChunkPacket {
    pub chunk_length: Short,
    pub chunk_data: ByteArray,
    pub percent_complete: Byte,
}

impl S2CPacket for LevelDataChunkPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_short(self.chunk_length)?;
        writer.write_byte_array(&self.chunk_data)?;
        writer.write_byte(self.percent_complete)
    }

    fn id(&self) -> Byte {
        0x03
    }
}

#[derive(Debug)]
pub struct LevelFinalisePacket {
    pub x_size: Short,
    pub y_size: Short,
    pub z_size: Short,
}

impl S2CPacket for LevelFinalisePacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_short(self.x_size)?;
        writer.write_short(self.y_size)?;
        writer.write_short(self.z_size)
    }

    fn id(&self) -> Byte {
        0x04
    }
}

#[derive(Debug)]
pub struct SetBlockPacket {
    pub x: Short,
    pub y: Short,
    pub z: Short,
    pub block_type: Byte,
}

impl S2CPacket for SetBlockPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_short(self.x)?;
        writer.write_short(self.y)?;
        writer.write_short(self.z)?;
        writer.write_byte(self.block_type)
    }

    fn id(&self) -> Byte {
        0x06
    }
}

#[derive(Debug)]
pub struct SpawnPlayerPacket {
    pub player_id: SByte,
    pub player_name: PacketString,
    pub x: FShort,
    pub y: FShort,
    pub z: FShort,
    pub yaw: Byte,
    pub pitch: Byte,
}

impl S2CPacket for SpawnPlayerPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_packet_string(&self.player_name)?;
        writer.write_fshort(&self.x)?;
        writer.write_fshort(&self.y)?;
        writer.write_fshort(&self.z)?;
        writer.write_byte(self.yaw)?;
        writer.write_byte(self.pitch)
    }

    fn id(&self) -> Byte {
        0x07
    }
}

#[derive(Debug)]
pub struct PlayerTeleportPacket {
    pub player_id: SByte,
    pub x: FShort,
    pub y: FShort,
    pub z: FShort,
    pub yaw: Byte,
    pub pitch: Byte,
}

impl S2CPacket for PlayerTeleportPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_fshort(&self.x)?;
        writer.write_fshort(&self.y)?;
        writer.write_fshort(&self.z)?;
        writer.write_byte(self.yaw)?;
        writer.write_byte(self.pitch)
    }

    fn id(&self) -> Byte {
        0x08
    }
}

#[derive(Debug)]
pub struct PlayerPosOriUpdatePacket {
    pub player_id: SByte,
    pub delta_x: FByte,
    pub delta_y: FByte,
    pub delta_z: FByte,
    pub yaw: Byte,
    pub pitch: Byte,
}

impl S2CPacket for PlayerPosOriUpdatePacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_fbyte(&self.delta_x)?;
        writer.write_fbyte(&self.delta_y)?;
        writer.write_fbyte(&self.delta_z)?;
        writer.write_byte(self.yaw)?;
        writer.write_byte(self.pitch)
    }

    fn id(&self) -> Byte {
        0x09
    }
}

#[derive(Debug)]
pub struct PlayerPosUpdatePacket {
    pub player_id: SByte,
    pub delta_x: FByte,
    pub delta_y: FByte,
    pub delta_z: FByte,
}

impl S2CPacket for PlayerPosUpdatePacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_fbyte(&self.delta_x)?;
        writer.write_fbyte(&self.delta_y)?;
        writer.write_fbyte(&self.delta_z)
    }

    fn id(&self) -> Byte {
        0x0a
    }
}

#[derive(Debug)]
pub struct PlayerOriUpdatePacket {
    pub player_id: SByte,
    pub yaw: Byte,
    pub pitch: Byte,
}

impl S2CPacket for PlayerOriUpdatePacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_byte(self.yaw)?;
        writer.write_byte(self.pitch)
    }

    fn id(&self) -> Byte {
        0x0b
    }
}

#[derive(Debug)]
pub struct DespawnPlayerPacket {
    pub player_id: SByte,
}

impl S2CPacket for DespawnPlayerPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)
    }

    fn id(&self) -> Byte {
        0x0c
    }
}

#[derive(Debug)]
pub struct MessagePacket {
    pub player_id: SByte,
    pub message: PacketString,
}

impl S2CPacket for MessagePacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_packet_string(&self.message)
    }

    fn id(&self) -> Byte {
        0x0d
    }
}

#[derive(Debug)]
pub struct DisconnectPlayerPacket {
    pub disconnect_reason: PacketString,
}

impl S2CPacket for DisconnectPlayerPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_packet_string(&self.disconnect_reason)
    }

    fn id(&self) -> Byte {
        0x0e
    }
}

#[derive(Debug)]
pub struct UpdateUserTypePacket {
    pub user_type: Byte,
}

impl S2CPacket for UpdateUserTypePacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_byte(self.user_type)
    }

    fn id(&self) -> Byte {
        0x0f
    }
}
