use crate::game::movement::Movement;
use crate::game::player::Player;
use crate::game::rand::Rand;
use crate::game::screens::Screen;
use crate::{AppSystems, game};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use fastnoise_lite::FastNoiseLite;
use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use std::f32::consts::PI;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            observe_surrounding,
            enemy_sync_image,
            awaking,
            hunt_player_or_sleep,
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Component)]
pub struct Enemy {
    // the base radius that this enemy observes
    pub observe_radius: f32,
}

#[derive(Component)]
pub struct Sleeping;

#[derive(Component)]
pub struct Awake {
    pub since: Duration,
    pub reorient: Timer,
}

#[derive(Component)]
pub struct Awaking {
    // awake once the timer hits zero
    pub timer: Timer,
}

impl Awaking {
    pub fn new(delay_secs: f32) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(delay_secs), TimerMode::Once),
        }
    }
}

pub fn enemy_bundle(rand: &mut Rand, assets: &game::Assets, enemy: Enemy) -> impl Bundle {
    let radius = enemy.observe_radius;

    (
        enemy,
        Sleeping,
        Movement {
            velocity: Vec2::ZERO,
        },
        Sprite {
            image: assets.enemy.clone(),
            custom_size: Some(Vec2::splat(32.0)),
            color: Color::srgb(0.2, 0.2, 0.2),
            anchor: Anchor::Center,
            ..default()
        },
        children![
            Name::new("Radius"),
            Sprite {
                image: assets.radius.clone(),
                custom_size: Some(Vec2::splat(radius) * 2.0),
                color: Color::srgba(1.0, 1.0, 1.0, 0.01),
                anchor: Anchor::Center,
                ..default()
            },
            // slightly below the actual enemy
            Transform::from_xyz(0.0, 0.0, -0.1)
                .with_rotation(Quat::from_rotation_z(rand.gen_range(0.0..PI * 2.0))),
        ],
    )
}

const COLOR_AWAKE: Color = Color::srgba(1.0, 0.1, 0.1, 1.0);
const COLOR_SLEEPING: Color = Color::srgba(0.9, 0.9, 0.9, 0.75);

fn enemy_sync_image(
    time: Res<Time<Virtual>>,
    mut enemies: Query<(&mut Sprite, Option<&Awake>), With<Enemy>>,
) {
    let noise = FastNoiseLite::new();

    for (mut sprite, awake) in &mut enemies {
        let color = match awake {
            Some(awake) => {
                let age = time.elapsed_secs() - awake.since.as_secs_f32();
                let amount = noise.get_noise_2d(0.0, age);
                let alpha = amount.abs() * 0.2 + 0.8;
                COLOR_AWAKE.with_alpha(alpha)
            }

            None => COLOR_SLEEPING,
        };

        if sprite.color != color {
            sprite.color = color;
        }
    }
}

pub fn generate_positions(
    seed: u64,
    center: Vec2,
    min_radius: f32,
    max_radius: f32,
    clearance: f32,
    count: usize,
) -> Vec<Vec2> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let mut positions = Vec::with_capacity(count);
    while positions.len() < count {
        let x = rng.gen_range(-max_radius..max_radius);
        let y = rng.gen_range(-max_radius..max_radius);

        let offset = vec2(x, y);

        if !(min_radius..max_radius).contains(&offset.length()) {
            // out of the circle or to near to the center
            continue;
        }

        let pos = center + offset;
        if positions
            .iter()
            .any(|other| pos.distance(*other) < clearance)
        {
            // some other position is too near
            continue;
        }

        positions.push(pos);
    }

    positions
}

fn observe_surrounding(
    mut rand: ResMut<Rand>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &Enemy, &Transform), With<Sleeping>>,
    query_players: Query<&Transform, With<Player>>,
    query_runners: Query<&Transform, (With<Enemy>, With<Awake>)>,
) {
    enum Other<'a> {
        Player(&'a Transform),
        Runner(&'a Transform),
    }

    impl<'a> Other<'a> {
        fn position(&self) -> Vec2 {
            match self {
                Other::Player(tr) => tr.translation.xy(),
                Other::Runner(tr) => tr.translation.xy(),
            }
        }
    }

    let others: Vec<_> = query_players
        .iter()
        .map(Other::Player)
        .chain(query_runners.iter().map(Other::Runner))
        .collect();

    for (enemy_id, enemy, enemy_transform) in &mut enemies {
        // get the nearest entity to this one
        let Some(other) = others.iter().min_by_key(|other| {
            OrderedFloat(other.position().distance(enemy_transform.translation.xy()))
        }) else {
            continue;
        };

        // get distance to the player
        let offset = other.position() - enemy_transform.translation.xy();

        let (max_distance, delay_secs_range) = match other {
            Other::Player(_player) => (enemy.observe_radius, 2.0..3.0),
            Other::Runner(_runner) => (32.0, 0.5..1.0),
        };

        if offset.length() > max_distance {
            // too far away, skipping this one
            continue;
        }

        // wake the guy up and go into the direction of the player
        commands
            .entity(enemy_id)
            .remove::<Sleeping>()
            .insert(Awaking::new(rand.gen_range(delay_secs_range)));
    }
}

fn awaking(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Transform, &mut Awaking)>,
) {
    for (entity, mut transform, mut awaking) in &mut enemies {
        transform.rotation *= Quat::from_rotation_z(PI * time.delta_secs());

        if !awaking.timer.tick(time.delta()).just_finished() {
            continue;
        }

        commands.entity(entity).remove::<Awaking>().insert(Awake {
            since: time.elapsed(),
            reorient: Timer::default(),
        });
    }
}

fn hunt_player_or_sleep(
    mut rand: ResMut<Rand>,
    time: Res<Time<Virtual>>,
    mut enemies: Query<(&Transform, &mut Awake, &mut Movement), With<Enemy>>,
    players: Query<&Transform, With<Player>>,
) {
    let players: Vec<_> = players.iter().collect();

    for (enemy_transform, mut enemy_awake, mut enemy_movement) in &mut enemies {
        if !enemy_awake.reorient.tick(time.delta()).just_finished() {
            continue;
        }

        // re-init the timer to reorient later
        enemy_awake.reorient = Timer::new(
            Duration::from_secs_f32(rand.gen_range(1.0..2.0)),
            TimerMode::Once,
        );

        // get the player that is nearest
        let Some(player) = players.iter().min_by_key(|p| {
            OrderedFloat(
                p.translation
                    .xy()
                    .distance(enemy_transform.translation.xy()),
            )
        }) else {
            continue;
        };

        // get vector to target
        let target = player.translation.xy() + random_vec2(rand.as_mut()) * 32.0;
        let offset = target - enemy_transform.translation.xy();
        enemy_movement.velocity = offset.normalize() * rand.gen_range(100.0..120.0);
    }
}

fn random_vec2(rng: &mut impl Rng) -> Vec2 {
    loop {
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let vec = vec2(x, y);
        if vec.length_squared() > 1.0 {
            continue;
        }

        break vec;
    }
}
