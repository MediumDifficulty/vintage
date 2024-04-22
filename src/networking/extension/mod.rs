use super::{
    c2s::{C2SPacket, C2SPacketEntry, PacketReader},
    listener::ClientInfo,
    s2c::{PacketWriter, S2CPacket},
    Byte, PacketString, Short,
};
use anyhow::Result;
use evenio::world::World;

pub mod s2c;

pub type Int = i32;

#[derive(Debug)]
pub struct ExtInfoPacket {
    pub app_name: PacketString,
    pub extension_count: Short,
}

impl S2CPacket for ExtInfoPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_packet_string(&self.app_name)?;
        writer.write_short(self.extension_count)
    }

    fn id(&self) -> u8 {
        0x10
    }
}

impl C2SPacket for ExtInfoPacket {
    fn exec(&self, world: &mut World, client_info: &ClientInfo) -> Result<()> {
        todo!()
    }
}

impl C2SPacketEntry for ExtInfoPacket {
    const ID: Byte = 0x10;
    const SIZE: usize = 67;

    fn deserialise(reader: &mut PacketReader) -> Result<Box<dyn C2SPacket>> {
        let app_name = reader.read_packet_string()?;
        let extension_count = reader.read_short()?;

        Ok(Box::new(ExtInfoPacket {
            app_name,
            extension_count,
        }))
    }
}

#[derive(Debug)]
pub struct ExtEntryPacket {
    pub ext_name: PacketString,
    pub version: Int,
}

impl S2CPacket for ExtEntryPacket {
    fn serialise(&self, writer: &mut PacketWriter) -> Result<()> {
        writer.write_packet_string(&self.ext_name)?;
        writer.write_int(self.version)
    }

    fn id(&self) -> u8 {
        0x11
    }
}
