use evenio::event::Event;
use glam::{IVec3, Vec3};

use crate::world::Block;

#[derive(Debug, Event)]
pub struct PlayerJoinEvent {
    pub username: String,
}

#[derive(Debug, Event)]
pub struct SetBlockEvent {
    pos: IVec3,
    placed: bool,
    block: Block,
}

#[derive(Debug, Event)]
pub struct PlayerMoveEvent {
    player_name: String,
    pos: Vec3,
}

#[derive(Debug, Event)]
pub struct PlayerMessageEvent {
    player_name: String,
    message: String,
}