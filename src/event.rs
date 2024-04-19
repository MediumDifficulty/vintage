use evenio::{entity::EntityId, event::Event};
use glam::{UVec3, Vec3};

use crate::world::{Block, PlayerId};

#[derive(Debug, Event)]
pub struct PlayerJoinEvent {
    pub entity_id: EntityId,
    pub username: String,
}

#[derive(Debug, Event)]
pub struct SetBlockEvent {
    pub pos: UVec3,
    pub placed: bool,
    pub block: Block,
}

#[derive(Debug, Event)]
pub struct PlayerMoveEvent {
    pub player_id: PlayerId,
    pub pos: Vec3,
}

#[derive(Debug, Event)]
pub struct PlayerMessageEvent {
    pub player_id: PlayerId,
    pub message: String,
}
