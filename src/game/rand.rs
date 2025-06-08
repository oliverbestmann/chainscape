use bevy::app::App;
use bevy::math::{FloatPow, Vec2, vec2};
use bevy::prelude::Resource;
use fastnoise_lite::FastNoiseLite;
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

pub struct Occupied {
    position: Vec2,
    clearance: f32,
}

pub struct Generate {
    pub center: Vec2,
    pub min_radius: f32,
    pub max_radius: f32,
    occupied: Vec<Occupied>,
}

pub fn weighted_by_noise(rand: &mut Rand, noise: FastNoiseLite) -> impl FnMut(f32) -> Vec2 {
    move |radius: f32| {
        loop {
            let random: f32 = rand.random();

            let candidate = rand.vec2() * radius;
            let noise_value = (noise.get_noise_2d(candidate.x, candidate.y) + 1.0).min(1.0);

            if random <= noise_value.squared() {
                return candidate;
            }
        }
    }
}

impl Generate {
    pub fn new(max_radius: f32, min_radius: f32, center: Vec2) -> Self {
        Self {
            center,
            min_radius,
            max_radius,
            occupied: Vec::new(),
        }
    }

    pub fn generate<Fn>(&mut self, mut random_point: Fn, count: usize, clearance: f32) -> Vec<Vec2>
    where
        Fn: FnMut(f32) -> Vec2,
    {
        let mut positions = Vec::with_capacity(count);

        while positions.len() < count {
            let offset = random_point(self.max_radius);

            if !(self.min_radius..self.max_radius).contains(&offset.length()) {
                // out of the circle or to near to the center
                continue;
            }

            let pos = self.center + offset;
            if self
                .occupied
                .iter()
                .any(|other| pos.distance(other.position) < clearance.max(other.clearance))
            {
                // some other point is too near
                continue;
            }

            positions.push(pos);

            // put into context
            self.occupied.push(Occupied {
                position: pos,
                clearance,
            })
        }

        positions
    }
}
