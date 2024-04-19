use std::{str::FromStr, sync::Arc, thread};

use anyhow::Result;
use evenio::prelude::*;
use glam::{uvec3, vec3, Vec3};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, Level};
use vintage::{
    event::{PlayerJoinEvent, SetBlockEvent},
    networking::{
        self, listener::{self, ClientPacket}, s2c::{self, PlayerTeleportPacket, S2CPacket, ServerIdentPacket}, FShort, PacketString, Short
    },
    world::{self, Block, BlockWorld, ClientConnection, Player, PlayerIdAllocator, Position, Rotation},
};

enum WorldEvent {
    Tick,
    Packet(ClientPacket),
}

#[derive(Event)]
struct TickEvent;

#[derive(Component)]
struct PacketBroadcaster(Arc<broadcast::Sender<Arc<Box<dyn S2CPacket>>>>);

#[derive(Component)]
struct PlayerSpawnLocation {
    position: Vec3,
    pitch: f32,
    yaw: f32,
}

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
    world.add_handler(set_block_handler);
    world.add_handler(player_spawn_handler);

    let player_id_allocator = world.spawn();
    world.insert(player_id_allocator, PlayerIdAllocator::new_empty());

    let player_spawn_location = world.spawn();
    world.insert(
        player_spawn_location,
        PlayerSpawnLocation {
            position: vec3(16.0, 20.0, 16.0),
            pitch: 0.,
            yaw: 0.
        }
    );

    let block_world = world.spawn();
    world.insert(
        block_world,
        BlockWorld::new(uvec3(64, 32, 64), |dims, world| {
            for x in 0..dims.x {
                world.set_block(uvec3(x, 0, 0), world::Block::RedCloth);
            }

            for y in 0..dims.y {
                world.set_block(uvec3(0, y, 0), world::Block::GreenCloth);
            }

            for z in 0..dims.z {
                world.set_block(uvec3(0, 0, z), world::Block::PurpleCloth);
            }

            for x in 0..dims.x {
                for z in 0..dims.z {
                    world.set_block(uvec3(x, 15, z), world::Block::Glass);
                }
            }
        }),
    );

    let (tx, mut rx) = mpsc::channel(32);
    let (broadcast_tx, _) = broadcast::channel(32);
    let broadcast_tx = Arc::new(broadcast_tx);

    let packet_broadcaster = world.spawn();
    world.insert(packet_broadcaster, PacketBroadcaster(broadcast_tx.clone()));

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
                    WorldEvent::Packet(packet) => {
                        if let Err(e) = packet.exec(&mut world) {
                            error!("failed to execute packet handler: {e}")
                        }
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
    // info!("Handling tick");
}

fn player_join_handler(
    e: Receiver<PlayerJoinEvent>,
    connections: Fetcher<&ClientConnection>,
    Single(block_world): Single<&BlockWorld>,
    Single(player_id_allocator): Single<&mut PlayerIdAllocator>,
    mut sender: Sender<(Insert<Player>, Insert<Position>, Insert<Rotation>)>,
    Single(spawn_location): Single<&PlayerSpawnLocation>,
) {
    debug!("Handling player join");
    let player_id = player_id_allocator.alloc();
    sender.insert(
        e.event.entity_id,
        Player {
            id: player_id,
            name: e.event.username.clone(),
        },
    );

    sender.insert(e.event.entity_id, Position(spawn_location.position));
    sender.insert(e.event.entity_id, Rotation {
        pitch: spawn_location.pitch,
        yaw: spawn_location.yaw,
    });

    let fetched = connections.get(e.event.entity_id).unwrap();
    info!("Player addr: {}", fetched.addr);

    fetched
        .sender
        .blocking_send(Box::new(ServerIdentPacket {
            protocol_version: 7,
            server_name: PacketString::from_str("vintage").unwrap(),
            server_motd: PacketString::from_str("Vintage server").unwrap(),
            user_type: 0x64,
        }))
        .unwrap();

    s2c::util::send_world(block_world, &fetched.sender).unwrap();
    
    fetched
        .sender
        .blocking_send(Box::new(PlayerTeleportPacket {
            player_id: -1,
            pitch: 0,
            yaw: 0,
            x: FShort::from(spawn_location.position.x),
            y: FShort::from(spawn_location.position.y),
            z: FShort::from(spawn_location.position.z),
        })).unwrap();

    fetched
        .sender
        .blocking_send(Box::new(s2c::SpawnPlayerPacket {
            player_id: -1,
            player_name: PacketString::from_str(&e.event.username).unwrap(),
            x: FShort::from(spawn_location.position.x),
            y: FShort::from(spawn_location.position.y),
            z: FShort::from(spawn_location.position.z),
            yaw: networking::util::to_angle_byte(spawn_location.yaw),
            pitch: networking::util::to_angle_byte(spawn_location.pitch),
        })).unwrap();
    
    debug!("Finished handling")
}

fn player_spawn_handler(
    e: Receiver<Insert<Player>, EntityId>,
    clients: Fetcher<(&ClientConnection, With<&Player>)>,
    Single(spawn_location): Single<&PlayerSpawnLocation>,
) {
    debug!("Handling player spawn");

    for (connection, _) in clients.iter() {
        connection.sender.blocking_send(Box::new(s2c::SpawnPlayerPacket {
            player_id: e.event.component.id,
            player_name: PacketString::from_str(&e.event.component.name).unwrap(),
            x: FShort::from(spawn_location.position.x),
            y: FShort::from(spawn_location.position.y),
            z: FShort::from(spawn_location.position.z),
            pitch: networking::util::to_angle_byte(spawn_location.pitch),
            yaw: networking::util::to_angle_byte(spawn_location.yaw),
        })).unwrap();
    }
}

fn set_block_handler(
    e: Receiver<SetBlockEvent>,
    Single(block_world): Single<&mut BlockWorld>,
    Single(broadcaster): Single<&PacketBroadcaster>,
) {
    let block = if e.event.placed {
        e.event.block as u8
    } else {
        Block::Air as u8
    };

    block_world.set_block(e.event.pos, e.event.block);

    broadcaster
        .0
        .send(Arc::new(Box::new(s2c::SetBlockPacket {
            block_type: block,
            x: e.event.pos.x as Short,
            y: e.event.pos.y as Short,
            z: e.event.pos.z as Short,
        })))
        .unwrap();
}
