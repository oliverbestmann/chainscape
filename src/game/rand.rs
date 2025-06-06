use bevy::app::App;
use bevy::math::{Vec2, vec2};
use bevy::prelude::Resource;
use rand::{Rng, RngCore, SeedableRng};

#[derive(Resource)]
pub struct Rand(rand::rngs::SmallRng);

impl RngCore for Rand {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }
}

impl Rand {
    /// Returns a random vec2 within the unit circle.
    pub fn vec2(&mut self) -> Vec2 {
        loop {
            let x = self.random_range(-1.0..1.0);
            let y = self.random_range(-1.0..1.0);
            let vec = vec2(x, y);
            if vec.length_squared() > 1.0 {
                continue;
            }

            break vec;
        }
    }
}

pub fn plugin(app: &mut App) {
    let r = rand::rngs::SmallRng::seed_from_u64(1);
    app.insert_resource(Rand(r));
}
