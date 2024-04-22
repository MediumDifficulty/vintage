use evenio::prelude::*;
use tracing::info;

use crate::{event::PlayerJoinEvent, world::ClientConnection};

pub fn add_cpe_handlers(world: &mut World) {
    world.add_handler(on_player_join);
}

fn on_player_join(e: Receiver<PlayerJoinEvent>, connections: Fetcher<&ClientConnection>) {
    info!("Player supports CPE: {}", e.event.cpe);
    let player = connections.get(e.event.entity_id).unwrap();
}
