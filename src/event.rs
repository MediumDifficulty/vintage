use evenio::{entity::EntityId, event::Event};
use glam::{IVec3, Vec3};

use crate::world::{Block, PlayerId};

#[derive(Debug, Event)]
pub struct PlayerJoinEvent(pub EntityId);

#[derive(Debug, Event)]
pub struct SetBlockEvent {
    pos: IVec3,
    placed: bool,
    block: Block,
}

#[derive(Debug, Event)]
pub struct PlayerMoveEvent {
    player_id: PlayerId,
    pos: Vec3,
}

#[derive(Debug, Event)]
pub struct PlayerMessageEvent {
    player_id: PlayerId,
    message: String,
}
