use anyhow::Ok;
use anyhow::Result;
use glam::Vec3;
use tokio::sync::mpsc;
use tracing::debug;

use crate::networking::util::to_angle_byte;
use crate::networking::FByte;
use crate::networking::FShort;
use crate::networking::Short;
use crate::world::BlockWorld;
use crate::world::PlayerId;
use crate::world::Rotation;

use super::LevelDataChunkPacket;
use super::LevelFinalisePacket;
use super::LevelInitPacket;
use super::S2CPacket;

const CHUNK_SIZE: usize = 1024;

pub fn send_world(world: &BlockWorld, sender: &mpsc::Sender<Box<dyn S2CPacket>>) -> Result<()> {
    sender.blocking_send(Box::new(LevelInitPacket {}))?;

    let serialised = world.serialise()?;

    for (i, chunk) in serialised.chunks(CHUNK_SIZE).enumerate() {
        let mut chunk_data = chunk.to_vec();
        chunk_data.resize(CHUNK_SIZE, 0);
        let chunk_data = chunk_data.try_into().unwrap();
        let percent_complete = ((i * CHUNK_SIZE * 100) / serialised.len()) as u8;

        sender.blocking_send(Box::new(LevelDataChunkPacket {
            chunk_length: chunk.len() as Short,
            chunk_data,
            percent_complete,
        }))?;
    }

    sender.blocking_send(Box::new(LevelFinalisePacket {
        x_size: world.dims().x as Short,
        y_size: world.dims().y as Short,
        z_size: world.dims().z as Short,
    }))?;

    Ok(())
}

/// # Args
/// `teleport_threshold` is the number of blocks the player needs to have moved to warrant the use of a [`super::PlayerTeleportPacket`]
///
/// pos and rot 1 are the original positions and rotations of the player
///
/// pos and rot 2 are the new positions and rotations of the player
pub fn send_player_move_packet(
    pos1: Vec3,
    pos2: Vec3,
    rot1: Rotation,
    rot2: Rotation,
    teleport_threshold: f32,
    player_id: PlayerId,
    sender: &mpsc::Sender<Box<dyn S2CPacket>>,
) -> Result<()> {
    let delta_distance = pos1.distance(pos2);
    let rotation_changed = rot1 != rot2;
    let position_changed = pos1 != pos2;

    debug!("distance: {delta_distance} threshold: {teleport_threshold}");

    if delta_distance < teleport_threshold {
        let delta_pos = pos2 - pos1;

        if position_changed && rotation_changed {
            return Ok(
                sender.blocking_send(Box::new(super::PlayerPosOriUpdatePacket {
                    player_id,
                    pitch: to_angle_byte(rot2.pitch),
                    yaw: to_angle_byte(rot2.yaw),
                    delta_x: FByte::from(delta_pos.x),
                    delta_y: FByte::from(delta_pos.y),
                    delta_z: FByte::from(delta_pos.z),
                }))?,
            );
        }

        if rotation_changed {
            return Ok(sender.blocking_send(Box::new(super::PlayerOriUpdatePacket {
                player_id,
                pitch: to_angle_byte(rot2.pitch),
                yaw: to_angle_byte(rot2.yaw),
            }))?);
        }

        if position_changed {
            return Ok(sender.blocking_send(Box::new(super::PlayerPosUpdatePacket {
                player_id,
                delta_x: FByte::from(delta_pos.x),
                delta_y: FByte::from(delta_pos.y),
                delta_z: FByte::from(delta_pos.z),
            }))?);
        }

        return Ok(());
    }
    
    Ok(sender.blocking_send(Box::new(super::PlayerTeleportPacket {
        player_id,
        pitch: to_angle_byte(rot2.pitch),
        yaw: to_angle_byte(rot2.yaw),
        x: FShort::from(pos2.x),
        y: FShort::from(pos2.y),
        z: FShort::from(pos2.z),
    }))?)
}
