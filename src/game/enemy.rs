use crate::game::movement::Movement;
use crate::game::player::Player;
use crate::game::screens::Screen;
use crate::{game, AppSystems};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::{Rng, SeedableRng};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (observe_surrounding, enemy_sync_image)
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

pub fn enemy_bundle(assets: &game::Assets, enemy: Enemy) -> impl Bundle {
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
                custom_size: Some(Vec2::splat(radius)),
                color: Color::srgba(1.0, 1.0, 1.0, 0.02),
                anchor: Anchor::Center,
                ..default()
            },
            // slightly below the actual enemy
            Transform::from_xyz(0.0, 0.0, -0.1),
        ],
    )
}

const COLOR_AWAKE: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
const COLOR_SLEEPING: Color = Color::srgba(0.9, 0.9, 0.9, 1.0);

fn enemy_sync_image(mut enemies: Query<(&mut Sprite, Option<&Sleeping>), With<Enemy>>) {
    for (mut sprite, sleeping) in &mut enemies {
        let color = if sleeping.is_some() {
            COLOR_SLEEPING
        } else {
            COLOR_AWAKE
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
    mut commands: Commands,
    mut enemies: Query<(Entity, &Transform, &mut Movement), With<Enemy>>,
    players: Query<&Transform, With<Player>>,
) {
    // should be just one player
    for player in &players {
        for (enemy_id, enemy_transform, mut enemy_movement) in &mut enemies {
            // get distance to the player
            let offset = player.translation.xy() - enemy_transform.translation.xy();

            if offset.length() > 128.0 {
                // too far awy
                continue;
            }

            // wake the guy up and go into the direction of the player
            commands.entity(enemy_id).remove::<Sleeping>();

            // start moving in the direction of the player, but be slightly faster
            // than he player is.
            enemy_movement.velocity = offset.normalize() * 110.0;
        }
    }
}
