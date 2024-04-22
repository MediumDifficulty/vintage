use std::net::SocketAddr;

use evenio::{entity::EntityId, event::Event};
use glam::{UVec3, Vec3};

use crate::world::{Block, Rotation};

#[derive(Debug, Event)]
pub struct PlayerJoinEvent {
    pub entity_id: EntityId,
    pub username: String,
    pub cpe: bool,
}

#[derive(Debug, Event)]
pub struct SetBlockEvent {
    pub pos: UVec3,
    pub placed: bool,
    pub block: Block,
}

#[derive(Debug, Event)]
pub struct PlayerMoveEvent {
    pub entity_id: EntityId,
    pub pos: Vec3,
    pub rot: Rotation,
}

#[derive(Debug, Event)]
pub struct PlayerMessageEvent {
    pub entity_id: EntityId,
    pub message: String,
}

#[derive(Debug, Event)]
pub struct PlayerDisconnectEvent(pub SocketAddr);
