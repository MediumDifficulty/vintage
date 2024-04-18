use std::{str::FromStr, thread};

use anyhow::Result;
use evenio::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, info, Level};
use vintage::{
    event::PlayerJoinEvent,
    networking::{
        listener::{self, ClientPacket},
        s2c::ServerIdent,
        PacketString,
    },
    world::ClientConnection,
};

enum WorldEvent {
    Tick,
    Packet(ClientPacket),
}

#[derive(Event)]
struct TickEvent;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(Level::TRACE)
        .init();

    info!("Starting");

    let mut world = World::new();

    world.add_handler(tick_handler);
    world.add_handler(player_join_handler);

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(listener::listen("127.0.0.1:8080", tx));
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    let (world_tx, mut world_rx) = mpsc::channel::<WorldEvent>(100);

    thread::Builder::new()
        .name("world".into())
        .spawn(move || {
            while let Some(event) = world_rx.blocking_recv() {
                match event {
                    WorldEvent::Tick => {
                        world.send(TickEvent {});
                    },
                    WorldEvent::Packet(packet) => {
                        packet.exec(&mut world);
                    },
                }
            }
        })
        .unwrap();

    loop {
        tokio::select! {
            Some(packet) = rx.recv() => {
                world_tx.send(WorldEvent::Packet(packet)).await?;
            }
            _ = interval.tick() => {
                world_tx.send(WorldEvent::Tick).await?;
            }
        }
    }
}

fn tick_handler(_: Receiver<TickEvent>) {
    info!("Handling tick");
}

fn player_join_handler(e: Receiver<PlayerJoinEvent>, fetcher: Fetcher<&ClientConnection>) {
    debug!("Handling player join");
    let fetched = fetcher.get(e.event.0).unwrap();
    info!("Player addr: {}", fetched.addr);

    fetched
        .sender
        .blocking_send(Box::new(ServerIdent {
            protocol_version: 7,
            server_name: PacketString::from_str("vintage").unwrap(),
            server_motd: PacketString::from_str("Vintage server").unwrap(),
            user_type: 0x64,
        }))
        .unwrap();
}
