use std::{io::{Read, Write}, net::SocketAddr};

use byteorder::{BigEndian, WriteBytesExt};
use enum_primitive::FromPrimitive;
use evenio::component::Component;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use glam::{UVec3, Vec3};
use tokio::sync::mpsc;
use anyhow::Result;

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
    name: String,
    id: PlayerId,
}

#[derive(Component, Debug)]
pub struct ClientConnection {
    pub sender: mpsc::Sender<Box<dyn S2CPacket>>,
    pub addr: SocketAddr,
}

#[derive(Component)]
pub struct Position(Vec3);

#[derive(Component)]
pub struct Rotation {
    pitch: f32,
    yaw: f32,
}

#[derive(Component)]
pub struct BlockWorld {
    dimensions: UVec3,
    blocks: Vec<Block>,
}

impl BlockWorld {
    pub fn new<F: FnOnce(UVec3, &mut Self)>(dimensions: UVec3, generator: F) -> Self {
        let mut world = Self {
            dimensions,
            blocks: vec![Block::Air; dimensions.x as usize * dimensions.y as usize * dimensions.z as usize],
        };

        generator(dimensions, &mut world);

        world
    }

    pub fn get_block(&self, pos: UVec3) -> Block {
        self.blocks[self.pos_to_index(pos)]
    }

    pub fn set_block(&mut self, pos: UVec3, block: Block) {
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
        let mut buffer = Vec::with_capacity(dimensions.x as usize * dimensions.y as usize * dimensions.z as usize);
        data.read_to_end(&mut buffer)?;

        let blocks = buffer
            .iter()
            .map(|&block| Block::from_u8(block).unwrap())
            .collect::<Vec<_>>();

        Ok(Self {
            dimensions,
            blocks,
        })
    }

    pub fn dims(&self) -> UVec3 {
        self.dimensions
    }
}
