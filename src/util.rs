use std::time::{Duration, Instant};

use evenio::{component::Component, event::Receiver, fetch::Single, world::World};
use tracing::info;

use crate::world::{BlockWorld, TickEvent};

#[derive(Component)]
struct WorldSaver {
    interval: Duration,
    last_save: Instant,
    save_path: String,
}

pub fn add_periodic_saver(world: &mut World, interval: Duration, save_path: &str) {
    let saver = world.spawn();
    world.insert(
        saver,
        WorldSaver {
            interval,
            last_save: Instant::now(),
            save_path: save_path.into(),
        },
    );

    world.add_handler(tick_handler);
}

#[allow(private_interfaces)]
pub fn tick_handler(
    _: Receiver<TickEvent>,
    Single(saver): Single<&mut WorldSaver>,
    Single(world): Single<&BlockWorld>,
) {
    if saver.last_save.elapsed() >= saver.interval {
        saver.last_save = Instant::now();
        // TODO: This might be good if it was on another thread
        world.save_to_file(saver.save_path.as_str()).unwrap();

        info!("Saved world")
    }
}
