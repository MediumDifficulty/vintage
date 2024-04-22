use std::{str::FromStr, sync::Arc};

use evenio::prelude::*;
use tokio::sync::broadcast;
use tracing::{debug, info};

use crate::{
    event::{
        PlayerDisconnectEvent, PlayerJoinEvent, PlayerMessageEvent, PlayerMoveEvent, SetBlockEvent,
    },
    networking::{
        self, c2s,
        s2c::{self, S2CPacket},
        ClientPacketRegistry, FShort, PacketString, Short,
    },
    world::{Block, BlockWorld, ClientConnection, Player, PlayerIdAllocator, Position, Rotation},
};

use self::config::PlayerSpawnLocation;

pub fn add_default_handlers(
    world: &mut World,
    broadcaster: Arc<broadcast::Sender<Arc<Box<dyn S2CPacket>>>>,
) {
    info!("Initialising default server configuration...");

    world.add_handler(player_join_handler.low());
    world.add_handler(set_block_handler.low());
    world.add_handler(player_spawn_handler.low());
    world.add_handler(player_disconnect_handler.low());
    world.add_handler(player_despawn_handler.low());
    world.add_handler(player_move_handler.low());
    world.add_handler(player_message_handler.low());

    let player_id_allocator = world.spawn();
    world.insert(player_id_allocator, PlayerIdAllocator::new_empty());

    let packet_broadcaster = world.spawn();
    world.insert(packet_broadcaster, PacketBroadcaster(broadcaster));
}

pub fn add_default_packets(registry: &mut ClientPacketRegistry) {
    registry.register::<c2s::PlayerIdentPacket>();
    registry.register::<c2s::SetBlockPacket>();
    registry.register::<c2s::PositionPacket>();
    registry.register::<c2s::MessagePacket>();
}

pub mod config {
    use evenio::prelude::*;
    use glam::Vec3;

    #[derive(Component)]
    pub struct PlayerSpawnLocation {
        pub position: Vec3,
        pub pitch: f32,
        pub yaw: f32,
    }
}

#[derive(Component)]
struct PacketBroadcaster(Arc<broadcast::Sender<Arc<Box<dyn S2CPacket>>>>);

fn player_join_handler(
    e: Receiver<PlayerJoinEvent>,
    connections: Fetcher<&ClientConnection>,
    players: Fetcher<(&Position, &Rotation, &Player)>,
    Single(block_world): Single<&BlockWorld>,
    Single(player_id_allocator): Single<&mut PlayerIdAllocator>,
    mut sender: Sender<(Insert<Player>, Insert<Position>, Insert<Rotation>)>,
    Single(spawn_location): Single<&PlayerSpawnLocation>,
) {
    let player_id = player_id_allocator.alloc(e.event.entity_id);
    sender.insert(
        e.event.entity_id,
        Player {
            id: player_id,
            name: e.event.username.clone(),
        },
    );

    sender.insert(e.event.entity_id, Position(spawn_location.position));
    sender.insert(
        e.event.entity_id,
        Rotation {
            pitch: spawn_location.pitch,
            yaw: spawn_location.yaw,
        },
    );

    let player = connections.get(e.event.entity_id).unwrap();
    info!("Player addr: {}", player.addr);

    player
        .sender
        .blocking_send(Box::new(s2c::ServerIdentPacket {
            protocol_version: 7,
            server_name: PacketString::from_str("vintage").unwrap(),
            server_motd: PacketString::from_str("Vintage server").unwrap(),
            user_type: 0x64,
        }))
        .unwrap();

    s2c::util::send_world(block_world, &player.sender).unwrap();

    player
        .sender
        .blocking_send(Box::new(s2c::PlayerTeleportPacket {
            player_id: -1,
            pitch: 0,
            yaw: 0,
            x: FShort::from(spawn_location.position.x),
            y: FShort::from(spawn_location.position.y),
            z: FShort::from(spawn_location.position.z),
        }))
        .unwrap();

    player
        .sender
        .blocking_send(Box::new(s2c::SpawnPlayerPacket {
            player_id: -1,
            player_name: PacketString::from_str(&e.event.username).unwrap(),
            x: FShort::from(spawn_location.position.x),
            y: FShort::from(spawn_location.position.y),
            z: FShort::from(spawn_location.position.z),
            yaw: networking::util::to_angle_byte(spawn_location.yaw),
            pitch: networking::util::to_angle_byte(spawn_location.pitch),
        }))
        .unwrap();

    // Populate world with other players
    for (pos, rot, other_player) in players.iter() {
        player
            .sender
            .blocking_send(Box::new(s2c::SpawnPlayerPacket {
                x: FShort::from(pos.0.x),
                y: FShort::from(pos.0.y),
                z: FShort::from(pos.0.z),
                pitch: networking::util::to_angle_byte(rot.pitch),
                yaw: networking::util::to_angle_byte(rot.yaw),
                player_id: other_player.id,
                player_name: PacketString::from_str(&other_player.name).unwrap(),
            }))
            .unwrap();
    }
}

fn player_spawn_handler(
    e: Receiver<Insert<Player>, EntityId>,
    clients: Fetcher<(&ClientConnection, With<&Player>)>,
    Single(spawn_location): Single<&PlayerSpawnLocation>,
) {
    for (connection, _) in clients.iter() {
        connection
            .sender
            .blocking_send(Box::new(s2c::SpawnPlayerPacket {
                player_id: e.event.component.id,
                player_name: PacketString::from_str(&e.event.component.name).unwrap(),
                x: FShort::from(spawn_location.position.x),
                y: FShort::from(spawn_location.position.y),
                z: FShort::from(spawn_location.position.z),
                pitch: networking::util::to_angle_byte(spawn_location.pitch),
                yaw: networking::util::to_angle_byte(spawn_location.yaw),
            }))
            .unwrap();
    }
}

fn player_disconnect_handler(
    e: Receiver<PlayerDisconnectEvent>,
    clients: Fetcher<(EntityId, &ClientConnection, With<&Player>)>,
    mut sender: Sender<Despawn>,
) {
    for (id, connection, _) in clients.iter() {
        if connection.addr == e.event.0 {
            sender.despawn(id);
        }
    }
}

fn player_despawn_handler(
    e: Receiver<Despawn, With<&Player>>,
    Single(player_id_allocator): Single<&mut PlayerIdAllocator>,
    fetcher: Fetcher<(EntityId, &Player, &ClientConnection)>,
) {
    let (_, player, _) = fetcher.get(e.event.0).unwrap();

    info!("Player {} left", player.name);

    player_id_allocator.free(player.id);
    for (id, _, connection) in fetcher.iter() {
        if id != e.event.0 {
            connection
                .sender
                .blocking_send(Box::new(s2c::DespawnPlayerPacket {
                    player_id: player.id,
                }))
                .unwrap();
        }
    }
}

fn player_move_handler(
    e: Receiver<PlayerMoveEvent>,
    mut players: Fetcher<(&mut Position, &mut Rotation, &Player)>,
    connections: Fetcher<(EntityId, &ClientConnection)>,
    Single(player_id_allocator): Single<&mut PlayerIdAllocator>,
) {
    let (original_position, original_rotation, _) = players.get_mut(e.event.entity_id).unwrap();

    for (id, connection) in connections.iter() {
        if id != e.event.entity_id {
            s2c::util::send_player_move_packet(
                original_position.0,
                e.event.pos,
                *original_rotation,
                e.event.rot,
                3.,
                player_id_allocator
                    .get_player_id(e.event.entity_id)
                    .unwrap(),
                &connection.sender,
            )
            .unwrap();
        }
    }

    original_position.0 = e.event.pos;
    *original_rotation = e.event.rot;
}

fn set_block_handler(
    e: Receiver<SetBlockEvent>,
    Single(block_world): Single<&mut BlockWorld>,
    Single(broadcaster): Single<&PacketBroadcaster>,
) {
    let block = if e.event.placed {
        e.event.block
    } else {
        Block::Air
    };

    block_world.set_block(e.event.pos, block);

    broadcaster
        .0
        .send(Arc::new(Box::new(s2c::SetBlockPacket {
            block_type: block as u8,
            x: e.event.pos.x as Short,
            y: e.event.pos.y as Short,
            z: e.event.pos.z as Short,
        })))
        .unwrap();
}

fn player_message_handler(
    e: Receiver<PlayerMessageEvent>,
    Single(broadcaster): Single<&PacketBroadcaster>,
    Single(player_id_allocator): Single<&mut PlayerIdAllocator>,
    players: Fetcher<&Player>,
) {
    debug!("Handling player message");
    let player_id = player_id_allocator
        .get_player_id(e.event.entity_id)
        .unwrap();
    let player = players.get(e.event.entity_id).unwrap();

    info!("Player {}: {}", player.name, e.event.message);

    broadcaster
        .0
        .send(Arc::new(Box::new(s2c::MessagePacket {
            message: PacketString::from_str(
                format!("{}: {}", player.name, &e.event.message).as_str(),
            )
            .unwrap(),
            player_id,
        })))
        .unwrap();
}
