use enum_primitive::FromPrimitive;
use evenio::prelude::*;
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}, sync::mpsc};
use tracing::{debug, info, warn, Level};
use vintage::packet::{c2s::{C2SPacket, PacketReader}, ClientPacketID};
use anyhow::Result;

#[derive(Event)]
struct PacketEvent;

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

    world.add_handler(packet_handler);
    world.add_handler(tick_handler);

    let (tx, mut rx) = mpsc::channel(100);
    
    tokio::spawn(listener(tx));
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            Some(packet) = rx.recv() => {
                debug!("{packet:?}");
                packet.exec(&mut world);
            }
            _ = interval.tick() => {
                world.send(TickEvent {});
            }
        }
    }
}

async fn listener(tx: mpsc::Sender<Box<dyn C2SPacket>>) {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    info!("Listening");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_client(socket, tx.clone()));
    }
}

async fn handle_client(mut socket: TcpStream, tx: mpsc::Sender<Box<dyn C2SPacket>>) -> Result<()> {
    loop {
        let packet_id = socket.read_u8().await.unwrap();
        info!("Starting packet with id: {packet_id}");
        let packet_id = match ClientPacketID::from_u8(packet_id) {
            Some(packet_id) => packet_id,
            None => {
                warn!("Invalid packet_id");
                continue
            },
        };
        debug!("Packet ID: {packet_id:?}");
        let mut packet_buf = vec![0u8; packet_id.size()];
        socket.read_exact(&mut packet_buf).await?;

        let packet = packet_id.deserialise(&mut PacketReader::new(packet_buf))
            .unwrap();

        tx.send(packet).await?;
    }
}

fn packet_handler(_: Receiver<PacketEvent>) {
    info!("Handling packet");
}

fn tick_handler(_: Receiver<TickEvent>) {
    info!("Handling tick");
}