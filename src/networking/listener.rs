use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use enum_primitive::FromPrimitive;
use evenio::{entity::EntityId, world::World};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{broadcast, mpsc},
};
use tracing::{info, trace, warn};

use crate::networking::{c2s::PacketReader, s2c::PacketWriter, ClientPacketID};

use super::{c2s::C2SPacket, s2c::S2CPacket};

pub struct ClientInfo {
    pub packet_sender: mpsc::Sender<Box<dyn S2CPacket>>,
    pub addr: SocketAddr,
    pub player_id: Mutex<Option<EntityId>>,
}

pub enum ClientMessage {
    Packet(ClientPacket),
    Disconnect(SocketAddr),
}

pub struct ClientPacket {
    packet: Box<dyn C2SPacket>,
    client_info: Arc<ClientInfo>,
}

impl ClientPacket {
    pub fn exec(&self, world: &mut World) -> Result<()> {
        self.packet.exec(world, &self.client_info)
    }
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A,
    tx: mpsc::Sender<ClientMessage>,
    broadcaster: Arc<broadcast::Sender<Arc<Box<dyn S2CPacket>>>>,
) {
    let listener = TcpListener::bind(addr).await.unwrap();
    // let broadcaster = Arc::new(broadcaster);

    info!("Listening");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_client(
            socket,
            addr,
            tx.clone(),
            broadcaster.subscribe(),
        ));
    }
}

async fn handle_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: mpsc::Sender<ClientMessage>,
    mut broadcaster: broadcast::Receiver<Arc<Box<dyn S2CPacket>>>,
) -> Result<()> {
    info!("Incoming connection from: {addr}");

    let (sender, mut receiver) = mpsc::channel(16);

    let info = Arc::new(ClientInfo {
        packet_sender: sender,
        addr,
        player_id: Mutex::new(None),
    });

    loop {
        tokio::select! {
            packet = receiver.recv() => {
                if let Some(packet) = packet {
                    write_packet(&packet, &mut socket).await?;
                } else {
                    break;
                }
            }
            packet = broadcaster.recv() => {
                if let Ok(packet) = packet {
                    write_packet(packet.as_ref(), &mut socket).await?;
                } else {
                    break;
                }
            }
            packet_id = socket.read_u8() => {
                if let Ok(packet_id) = packet_id {
                    let packet_id = match ClientPacketID::from_u8(packet_id) {
                        Some(packet_id) => packet_id,
                        None => {
                            warn!("Invalid packet ID: {packet_id}");
                            continue;
                        },
                    };

                    let mut packet_buf = vec![0u8; packet_id.size()];
                    socket.read_exact(&mut packet_buf).await?;

                    let packet = packet_id
                        .deserialise(&mut PacketReader::new(packet_buf))
                        .unwrap();


                    // TODO: use env variable to make this if configurable
                    if packet_id != ClientPacketID::Position {
                        trace!("Received packet: {packet:?}");
                    }

                    tx.send(ClientMessage::Packet(ClientPacket { packet, client_info: info.clone() })).await?;
                } else {
                    break;
                }
            }
        }
    }

    info!("Client disconnected");
    tx.send(ClientMessage::Disconnect(addr)).await?;

    Ok(())
}

// FIXME: Remove &Box
async fn write_packet(packet: &Box<dyn S2CPacket>, socket: &mut TcpStream) -> Result<()> {
    trace!("Sending packet: {:?}", packet);
    let mut writer = PacketWriter::new_with_capacity(1);
    writer.write_packet_boxed(packet)?;
    socket.write_all(&writer.into_inner()).await?;

    Ok(())
}
