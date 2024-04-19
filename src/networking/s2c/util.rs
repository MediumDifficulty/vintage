use anyhow::Ok;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::networking::Short;
use crate::world::BlockWorld;

use super::LevelDataChunk;
use super::LevelFinalise;
use super::LevelInit;
use super::S2CPacket;

const CHUNK_SIZE: usize = 1024;

pub fn send_world(world: &BlockWorld, sender: &mpsc::Sender<Box<dyn S2CPacket>>) -> Result<()> {
    sender.blocking_send(Box::new(LevelInit {}))?;

    let serialised = world.serialise()?;

    for (i, chunk) in serialised.chunks(CHUNK_SIZE).enumerate() {
        let mut chunk_data = chunk.to_vec();
        chunk_data.resize(CHUNK_SIZE, 0);
        let chunk_data = chunk_data.try_into().unwrap();
        let percent_complete = ((i * CHUNK_SIZE * 100) / serialised.len()) as u8;

        sender.blocking_send(Box::new(LevelDataChunk {
            chunk_length: chunk.len() as Short,
            chunk_data,
            percent_complete,
        }))?;
    }

    sender.blocking_send(Box::new(LevelFinalise {
        x_size: world.dims().x as Short,
        y_size: world.dims().y as Short,
        z_size: world.dims().z as Short,
    }))?;

    Ok(())
}