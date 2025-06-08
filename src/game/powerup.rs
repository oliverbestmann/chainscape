use crate::game;
use crate::game::enemy::{Awake, Enemy};
use crate::game::hud::AddScore;
use crate::game::movement::Movement;
use crate::game::player::Player;
use crate::game::rand::Rand;
use crate::game::screens::Screen;
use crate::game::squishy::Squishy;
use avian2d::prelude::{Collider, Collisions, Sensor};
use bevy::ecs::system::RunSystemOnce;
use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            collect_powerup,
            handle_delayed_explosions,
            explosion_fade_out,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
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

fn apply_powerup_coin(
    mut rand: ResMut<Rand>,
    mut player: Single<(&mut Player, &Transform)>,
    mut add_score: EventWriter<AddScore>,
) {
    let (player, player_transform) = &mut *player;

    // add bonus score
    let score = player.add_score(rand.random_range(3..=6) * 10);
    add_score.write(AddScore {
        score,
        position: player_transform.translation.xy(),
    });
}

fn apply_powerup_explosion(mut commands: Commands, player: Single<Entity, With<Player>>) {
    let label = commands
        .spawn((Text2d::new("Foobar"), Anchor::BottomLeft))
        .id();

    commands.entity(*player).insert(DelayedExplosion {
        timer: Timer::from_seconds(2.0, TimerMode::Once),
        label,
    });
}

fn handle_delayed_explosions(
    mut commands: Commands,
    mut rand: ResMut<Rand>,
    mut player: Single<(Entity, &mut DelayedExplosion, &mut Player, &Transform), Without<Text2d>>,
    mut label: Query<(&mut Text2d, &mut Transform)>,
    enemies: Query<(Entity, &Transform, Has<Awake>), (With<Enemy>, Without<Text2d>)>,
    assets: Res<game::Assets>,
    time: Res<Time>,
    mut add_score: EventWriter<AddScore>,
) {
    let (player_entity, explosion, player, player_transform) = &mut *player;

    if !explosion.timer.tick(time.delta()).just_finished() {
        if let Ok((mut text, mut transform)) = label.get_mut(explosion.label) {
            transform.translation.x = player_transform.translation.x + 24.0;
            transform.translation.y = player_transform.translation.y + 24.0;

            text.0 = format!("boom in {:1.2}s", explosion.timer.remaining_secs());
        }

        return;
    }

    // remove the label if it still exists
    commands.entity(explosion.label).try_despawn();

    // remove the scheduled explosion from the player
    commands.entity(*player_entity).remove::<DelayedExplosion>();

    let blast_radius = rand.random_range(200.0..300.0);

    for (enemy, enemy_transform, enemy_is_awake) in enemies {
        let distance = enemy_transform
            .translation
            .xy()
            .distance(player_transform.translation.xy());

        if distance > blast_radius {
            continue;
        }

        // kill enemy
        commands.entity(enemy).despawn();

        add_score.write(AddScore {
            score: player.add_kill(enemy_is_awake),
            position: enemy_transform.translation.xy(),
        });
    }

    // spawn an explosion circle
    commands.spawn((
        Name::new("Explosion"),
        StateScoped(Screen::Gameplay),
        Transform::from_translation(player_transform.translation.with_z(0.0)),
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
struct DelayedExplosion {
    timer: Timer,
    label: Entity,
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
