use evenio::component::Component;
use glam::Vec3;
use tokio::sync::mpsc;

use crate::packet::s2c::S2CPacket;

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
struct Player {
    name: String,
    id: PlayerId,
}

#[derive(Component)]
struct ClientConnection {
    sender: mpsc::Sender<Box<dyn S2CPacket>>,
}

#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Rotation {
    pitch: f32,
    yaw: f32,
}
