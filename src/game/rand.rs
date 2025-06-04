use bevy::app::App;
use bevy::prelude::Resource;
use rand::{Error, RngCore, SeedableRng};

#[derive(Resource)]
pub struct Rand(pub rand::rngs::StdRng);

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

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0.try_fill_bytes(dest)
    }
}

pub fn plugin(app: &mut App) {
    let r = rand::rngs::StdRng::seed_from_u64(1);
    app.insert_resource(Rand(r));
}
