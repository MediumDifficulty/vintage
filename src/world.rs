use std::{
    fs::{self, File},
    io::{Cursor, Read, Write},
    net::SocketAddr,
    ops::Sub,
};

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use enum_primitive::FromPrimitive;
use evenio::{component::Component, entity::EntityId, event::Event};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use glam::{UVec3, Vec3};
use tokio::sync::mpsc;
use tracing::debug;

use crate::networking::s2c::S2CPacket;

enum_from_primitive! {
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Stone,
    GrassBlock,
    Dirt,
    Cobblestone,
    Planks,
    Sapling,
    Bedrock,
    FlowingWater,
    StationaryWater,
    FlowingLava,
    StationaryLava,
    Sand,
    Gravel,
    GoldOre,
    IronOre,
    CoalOre,
    Woord,
    Leaves,
    Sponge,
    Glass,
    RedCloth,
    OrangeCloth,
    YellowCloth,
    ChartreuseCloth,
    GreenCloth,
    SpringGreenCloth,
    CyanCloth,
    CapriCloth,
    UltramarineCloth,
    PurpleCloth,
    VioletCloth,
    MagentaCloth,
    RoseCloth,
    DarkGreyCloth,
    LightGreyCloth,
    WhiteCloth,
    Flower,
    Rose,
    BrownMushroom,
    RedMushroom,
    BlockOfGold,
    BlockOfIron,
    DoubleSlab,
    Slab,
    Bricks,
    TNT,
    Bookshelf,
    MossyCobbleStone,
    Obsidian,
}
}

pub type PlayerId = i8;

#[derive(Component)]
pub struct Player {
    pub name: String,
    pub id: PlayerId,
}

#[derive(Component, Debug)]
pub struct ClientConnection {
    pub sender: mpsc::Sender<Box<dyn S2CPacket>>,
    pub addr: SocketAddr,
}

#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}

impl Sub for Rotation {
    type Output = Rotation;

    fn sub(self, rhs: Self) -> Self::Output {
        Rotation {
            pitch: self.pitch - rhs.pitch,
            yaw: self.yaw - rhs.yaw,
        }
    }
}

#[derive(Component)]
pub struct TickRate(pub u32);

#[derive(Event)]
pub struct TickEvent;

#[derive(Component)]
pub struct BlockWorld {
    dimensions: UVec3,
    blocks: Vec<Block>,
}

impl BlockWorld {
    pub fn new<F: FnOnce(UVec3, &mut Self)>(dimensions: UVec3, generator: F) -> Self {
        let mut world = Self {
            dimensions,
            blocks: vec![
                Block::Air;
                dimensions.x as usize * dimensions.y as usize * dimensions.z as usize
            ],
        };

        generator(dimensions, &mut world);

        world
    }

    pub fn get_block(&self, pos: UVec3) -> Block {
        self.blocks[self.pos_to_index(pos)]
    }

    pub fn set_block(&mut self, pos: UVec3, block: Block) {
        debug!("Setting block at: {pos:?}");
        let index = self.pos_to_index(pos);
        self.blocks[index] = block;
    }

    fn pos_to_index(&self, pos: UVec3) -> usize {
        (pos.x + pos.z * self.dims().z + pos.y * self.dims().x * self.dims().z) as usize
    }

    pub fn serialise(&self) -> Result<Vec<u8>> {
        let mut data = GzEncoder::new(
            Vec::with_capacity(
                (self.dimensions.x * self.dimensions.y * self.dimensions.z) as usize,
            ),
            Compression::default(),
        );

        data.write_i32::<BigEndian>(self.blocks.len() as i32)?;

        data.write_all(
            self.blocks
                .iter()
                .map(|&block| block as u8)
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .unwrap();

        Ok(data.finish()?)
    }

    pub fn deserialise(data: &[u8], dimensions: UVec3) -> Result<Self> {
        let mut data = GzDecoder::new(data);
        let mut buffer = Vec::with_capacity(
            dimensions.x as usize * dimensions.y as usize * dimensions.z as usize,
        );
        let block_amount = data.read_i32::<BigEndian>()?;
        data.read_to_end(&mut buffer)?;

        if block_amount as u32 != dimensions.x * dimensions.y * dimensions.z {
            return Err(anyhow::anyhow!("Invalid block amount"));
        }

        let blocks = buffer
            .iter()
            .map(|&block| Block::from_u8(block).unwrap())
            .collect::<Vec<_>>();

        Ok(Self { dimensions, blocks })
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let mut reader = File::open(path)?;
        let dim_x = reader.read_i16::<BigEndian>()?;
        let dim_y = reader.read_i16::<BigEndian>()?;
        let dim_z = reader.read_i16::<BigEndian>()?;

        let mut data = Vec::with_capacity(dim_x as usize * dim_y as usize * dim_z as usize);
        reader.read_to_end(&mut data)?;

        Self::deserialise(&data, UVec3::new(dim_x as u32, dim_y as u32, dim_z as u32))
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let mut writer = Cursor::new(Vec::with_capacity(self.blocks.len() + 6));
        writer.write_i16::<BigEndian>(self.dimensions.x as i16)?;
        writer.write_i16::<BigEndian>(self.dimensions.y as i16)?;
        writer.write_i16::<BigEndian>(self.dimensions.z as i16)?;

        let data = self.serialise()?;
        writer.write_all(&data)?;

        fs::write(path, writer.into_inner())?;

        Ok(())
    }

    pub fn new_or_load_from_file(
        path: &str,
        dimensions: UVec3,
        generator: impl FnOnce(UVec3, &mut Self),
    ) -> Self {
        if let Ok(world) = Self::load_from_file(path) {
            world
        } else {
            Self::new(dimensions, generator)
        }
    }

    pub fn dims(&self) -> UVec3 {
        self.dimensions
    }
}

#[derive(Component)]
pub struct PlayerIdAllocator {
    occupation: Vec<Option<EntityId>>,
}

impl PlayerIdAllocator {
    pub fn new_empty() -> Self {
        PlayerIdAllocator {
            occupation: vec![None; 127],
        }
    }

    pub fn alloc(&mut self, entity_id: EntityId) -> PlayerId {
        for (id, occupied) in self.occupation.iter_mut().enumerate() {
            if occupied.is_none() {
                *occupied = Some(entity_id);
                return id as PlayerId;
            }
        }

        panic!("No more player ids available");
    }

    pub fn free(&mut self, id: PlayerId) {
        self.occupation[id as usize] = None;
    }

    pub fn get_entity_id(&self, id: PlayerId) -> Option<EntityId> {
        self.occupation[id as usize]
    }

    pub fn get_player_id(&self, entity_id: EntityId) -> Option<PlayerId> {
        for (id, occupied) in self.occupation.iter().enumerate() {
            if let Some(occupation) = occupied {
                if *occupation == entity_id {
                    return Some(id as PlayerId);
                }
            }
        }

        None
    }
}
