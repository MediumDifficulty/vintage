use anyhow::Result;
use evenio::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, info, Level};
use vintage::{event::PlayerJoinEvent, networking::listener, world::{ClientConnection, Player}};


#[derive(Event)]
struct TickEvent;

#[tokio::main]
async fn main() {
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

    loop {
        tokio::select! {
            Some(packet) = rx.recv() => {
                packet.exec(&mut world);
            }
            _ = interval.tick() => {
                world.send(TickEvent {});
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
}