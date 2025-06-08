use crate::game;
use crate::game::enemy::Enemy;
use crate::game::movement::Movement;
use crate::game::player::Player;
use crate::game::screens::Screen;
use crate::game::squishy::Squishy;
use avian2d::prelude::{Collider, Collisions, Sensor};
use bevy::ecs::system::RunSystemOnce;
use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (collect_powerup, explosion_fade_out).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Copy, Clone, Component, Debug)]
pub enum Powerup {
    Speed,
    Explosion,
    Coin,
}

pub fn powerup_bundle(assets: &game::Assets, powerup: Powerup) -> impl Bundle {
    (
        powerup,
        Sprite {
            image: match powerup {
                Powerup::Speed => assets.up_speed.clone(),
                Powerup::Explosion => assets.up_explosion.clone(),
                Powerup::Coin => assets.up_coin.clone(),
            },
            color: Color::oklch(0.868, 0.174, 90.43),
            custom_size: Some(Vec2::splat(48.0)),
            anchor: Anchor::Center,
            ..default()
        },
        Squishy {
            frequency: 0.5,
            offset: Duration::ZERO,
            scale_min: Vec2::splat(0.8),
            scale_max: Vec2::splat(1.2),
        },
        Sensor,
        Collider::circle(24.0),
    )
}

pub struct ApplyPowerup(pub Powerup);

impl Command for ApplyPowerup {
    fn apply(self, world: &mut World) {
        let Self(powerup) = self;

        match powerup {
            Powerup::Speed => {
                _ = world.run_system_once(apply_powerup_speed);
            }
            Powerup::Explosion => {
                _ = world.run_system_once(apply_powerup_explosion);
            }
            Powerup::Coin => {
                _ = world.run_system_once(apply_powerup_coin);
            }
        }
    }
}

fn apply_powerup_speed(mut player: Single<&mut Movement, With<Player>>) {
    info!("Double the players speed until the next turn.");
    player.target_velocity *= 2.0;
}

fn apply_powerup_coin(mut player: Single<&mut Player>) {
    // add bonus score
    player.bonus_score += 10;
}

fn apply_powerup_explosion(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    assets: Res<game::Assets>,
) {
    let blast_radius = 256.0;

    for (enemy, enemy_transform) in enemies {
        let distance = enemy_transform
            .translation
            .xy()
            .distance(player.translation.xy());

        if distance > blast_radius {
            continue;
        }

        // kill enemy
        commands.entity(enemy).despawn();
    }

    // spawn an explosion circle
    commands.spawn((
        Name::new("Explosion"),
        StateScoped(Screen::Gameplay),
        Transform::from_translation(player.translation.with_z(0.0)),
        Explosion(Timer::from_seconds(0.25, TimerMode::Once)),
        Sprite {
            image: assets.circle.clone(),
            anchor: Anchor::Center,
            custom_size: Some(Vec2::splat(2.0 * blast_radius * 1.1)),
            color: Color::srgba(1.0, 1.0, 1.0, 0.75),
            ..default()
        },
    ));
}

#[derive(Component)]
struct Explosion(Timer);

fn explosion_fade_out(
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut Sprite, &mut Explosion)>,
    time: ResMut<Time<Virtual>>,
) {
    for (entity, mut sprite, mut explosion) in &mut explosions {
        if explosion.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let alpha = explosion.0.fraction_remaining().squared();
        sprite.color.set_alpha(alpha);
    }
}

fn collect_powerup(
    mut commands: Commands,
    collisions: Collisions,
    query_powerups: Query<(Entity, &Powerup)>,
    player: Single<Entity, With<Player>>,
) {
    for (powerup_entity, powerup) in &query_powerups {
        for collider in collisions.entities_colliding_with(powerup_entity) {
            if collider != *player {
                continue;
            }

            // apply entity to player
            commands.queue(ApplyPowerup(*powerup));

            // remove the powerup entity
            commands.entity(powerup_entity).despawn();
        }
    }
}
