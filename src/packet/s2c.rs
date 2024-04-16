use std::{fmt::Debug, io::{Cursor, Write}, str::FromStr};

use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};

use super::{Byte, ByteArray, FByte, FShort, PacketString, SByte, Short};

pub struct PacketWriter {
    buffer: Cursor<Vec<u8>>
}

impl PacketWriter {
    pub fn new(data: Vec<u8>) -> PacketWriter {
        PacketWriter {
            buffer: Cursor::new(data)
        }
    }

    pub fn new_empty() -> PacketWriter {
        PacketWriter {
            buffer: Cursor::new(Vec::new())
        }
    }

    pub fn new_with_capacity(capacity: usize) -> PacketWriter {
        PacketWriter {
            buffer: Cursor::new(Vec::with_capacity(capacity))
        }
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
}

pub trait S2CPacket: Send + Sync + Debug {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()>;
    fn id(&self) -> Byte;
}

#[derive(Debug)]
pub struct PlayerIdent {
    protocol_version: Byte,
    server_name: PacketString,
    server_motd: PacketString,
    user_type: Byte,
}

impl S2CPacket for PlayerIdent {
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
pub struct Ping;

impl S2CPacket for Ping {
    fn serialise(&self, _writer: &mut PacketWriter) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> Byte {
        0x01
    }
}

#[derive(Debug)]
pub struct LevelInit;

impl S2CPacket for LevelInit {
    fn serialise(&self, _writer: &mut PacketWriter) -> Result<()> {
        Ok(())
    }

    fn id(&self) -> Byte {
        0x02
    }
}

#[derive(Debug)]
pub struct LevelDataChunk {
    chunk_length: Short,
    chunk_data: ByteArray,
    percent_complete: Byte,
}

impl S2CPacket for LevelDataChunk {
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
pub struct LevelFinalise {
    x_size: Short,
    y_size: Short,
    z_size: Short,
}

impl S2CPacket for LevelFinalise {
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
pub struct SetBlock {
    x: Short,
    y: Short,
    z: Short,
    block_type: Byte
}

impl S2CPacket for SetBlock {
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
pub struct SpawnPlayer {
    player_id: SByte,
    player_name: PacketString,
    x: FShort,
    y: FShort,
    z: FShort,
}

impl S2CPacket for SpawnPlayer {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_packet_string(&self.player_name)?;
        writer.write_fshort(&self.x)?;
        writer.write_fshort(&self.y)?;
        writer.write_fshort(&self.z)
    }

    fn id(&self) -> Byte {
        0x07
    }
}

#[derive(Debug)]
pub struct PlayerTeleport {
    player_id: SByte,
    x: FShort,
    y: FShort,
    z: FShort,
    yaw: Byte,
    pitch: Byte,
}

impl S2CPacket for PlayerTeleport {
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
pub struct PlayerPosOriUpdate {
    player_id: SByte,
    delta_x: FShort,
    delta_y: FShort,
    delta_z: FShort,
    yaw: Byte,
    pitch: Byte,
}

impl S2CPacket for PlayerPosOriUpdate {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_fshort(&self.delta_x)?;
        writer.write_fshort(&self.delta_y)?;
        writer.write_fshort(&self.delta_z)?;
        writer.write_byte(self.yaw)?;
        writer.write_byte(self.pitch)
    }

    fn id(&self) -> Byte {
        0x09
    }
}

#[derive(Debug)]
pub struct PlayerPosUpdate {
    player_id: SByte,
    delta_x: FShort,
    delta_y: FShort,
    delta_z: FShort,
}

impl S2CPacket for PlayerPosUpdate {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_fshort(&self.delta_x)?;
        writer.write_fshort(&self.delta_y)?;
        writer.write_fshort(&self.delta_z)
    }

    fn id(&self) -> Byte {
        0x0a
    }
}

#[derive(Debug)]
pub struct PlayerOriUpdate {
    player_id: SByte,
    yaw: Byte,
    pitch: Byte,
}

impl S2CPacket for PlayerOriUpdate {
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
pub struct DespawnPlayer {
    player_id: SByte,
}

impl S2CPacket for DespawnPlayer {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)
    }

    fn id(&self) -> Byte {
        0x0c
    }
}

#[derive(Debug)]
pub struct Message {
    player_id: SByte,
    message: PacketString,
}

impl S2CPacket for Message {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_sbyte(self.player_id)?;
        writer.write_packet_string(&self.message)
    }

    fn id(&self) -> Byte {
        0x0d
    }
}

#[derive(Debug)]
pub struct DisconnectPlayer {
    disconnect_reason: PacketString,
}

impl S2CPacket for DisconnectPlayer {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_packet_string(&self.disconnect_reason)
    }

    fn id(&self) -> Byte {
        0x0e
    }
}

#[derive(Debug)]
pub struct UpdateUserType {
    user_type: Byte,
}

impl S2CPacket for UpdateUserType {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_byte(self.user_type)
    }

    fn id(&self) -> Byte {
        0x0f
    }
}