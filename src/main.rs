use std::{sync::Arc, thread, time::Duration};

use anyhow::Result;
use evenio::prelude::*;
use glam::{uvec3, vec3};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, Level};
use vintage::{
    default::{self, config::PlayerSpawnLocation}, event::PlayerDisconnectEvent, networking::listener::{self, ClientMessage}, util::add_periodic_saver, world::{Block, BlockWorld, TickEvent}
};

enum WorldEvent {
    Tick,
    ClientMessage(ClientMessage),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting");

    let mut world = World::new();

    let player_spawn_location = world.spawn();
    world.insert(
        player_spawn_location,
        PlayerSpawnLocation {
            position: vec3(16.0, 34.0, 16.0),
            pitch: 0.,
            yaw: 0.,
        },
    );

    let block_world = world.spawn();
    world.insert(
        block_world,
        BlockWorld::new_or_load_from_file("./level.bin", uvec3(128, 64, 128), |dims, world| {
            for x in 0..dims.x {
                for y in 0..31 {
                    for z in 0..dims.z {
                        world.set_block(uvec3(x, y, z), Block::Dirt);
                    }
                }
            }

            for x in 0..dims.x {
                for z in 0..dims.z {
                    world.set_block(uvec3(x, 32, z), Block::GrassBlock);
                }
            }
        }),
    );

    let (tx, mut rx) = mpsc::channel(32);
    let (broadcast_tx, _) = broadcast::channel(32);
    let broadcast_tx = Arc::new(broadcast_tx);

    default::add_default_handlers(&mut world, broadcast_tx.clone());
    add_periodic_saver(&mut world, Duration::from_secs(60), "./level.bin");

    tokio::spawn(listener::listen("127.0.0.1:8080", tx, broadcast_tx));
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    let (world_tx, mut world_rx) = mpsc::channel::<WorldEvent>(32);

    thread::Builder::new()
        .name("world".into())
        .spawn(move || {
            while let Some(event) = world_rx.blocking_recv() {
                match event {
                    WorldEvent::Tick => {
                        world.send(TickEvent {});
                    },
                    WorldEvent::ClientMessage(message) => match message {
                        ClientMessage::Packet(packet) => {
                            if let Err(e) = packet.exec(&mut world) {
                                error!("failed to execute packet handler: {e}")
                            }
                        },
                        ClientMessage::Disconnect(addr) => world.send(PlayerDisconnectEvent(addr)),
                    },
                }
            }
        })
        .unwrap();

    loop {
        tokio::select! {
            Some(packet) = rx.recv() => {
                world_tx.send(WorldEvent::ClientMessage(packet)).await?;
            }
            _ = interval.tick() => {
                world_tx.send(WorldEvent::Tick).await?;
            }
        }
    }
}

fn tick_handler(_: Receiver<TickEvent>) {
    // info!("Handling tick");
}
