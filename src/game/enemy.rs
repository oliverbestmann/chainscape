use crate::game::player::Player;
use crate::game::rand::Rand;
use crate::game::screens::Screen;
use crate::game::squishy::Squishy;
use crate::{AppSystems, game};
use avian2d::prelude::{
    AngularVelocity, Collider, ColliderDisabled, ExternalForce, LinearDamping, LinearVelocity,
    MaxLinearSpeed, RigidBody,
};
use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use fastnoise_lite::FastNoiseLite;
use ordered_float::OrderedFloat;
use rand::Rng;
use std::ops::Range;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            enable_disable_colliders_for_not_awake,
            state_sleeping,
            state_awaking,
            state_awake_hunt_player,
            state_awake_avoid_collisions,
            restrict_number_of_enemies_awake,
            enemy_sync_image,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Sleeping {
    pub when: Duration,
}

#[derive(Component)]
pub struct Awaking {
    // awake once the timer hits zero
    pub timer: Timer,
}

#[derive(Component)]
pub struct Awake {
    pub since: Duration,
    pub seed: f32,
    pub reorient: Timer,
}

impl Awaking {
    pub fn new(rand: &mut impl Rng, delay_secs_range: Range<f32>) -> Self {
        let delay_secs = rand.random_range(delay_secs_range);

        Self {
            timer: Timer::new(Duration::from_secs_f32(delay_secs), TimerMode::Once),
        }
    }
}

pub fn enemy_bundle(rand: &mut Rand, assets: &game::Assets) -> impl Bundle {
    (
        Enemy,
        Sleeping {
            when: Duration::ZERO,
        },
        Sprite {
            image: assets.enemy.clone(),
            custom_size: Some(Vec2::splat(48.0)),
            color: Color::srgb(0.2, 0.2, 0.2),
            anchor: Anchor::Center,
            ..default()
        },
        RigidBody::Dynamic,
        Collider::rectangle(20.0, 20.0),
        LinearVelocity::ZERO,
        MaxLinearSpeed(rand.random_range(100.0..140.0)),
        ExternalForce::ZERO.with_persistence(false),
        ColliderDisabled,
    )
}

fn enable_disable_colliders_for_not_awake(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    enemies_enabled: Query<(Entity, &Transform), (With<Sleeping>, Without<ColliderDisabled>)>,
    enemies_disabled: Query<(Entity, &Transform), (With<Sleeping>, With<ColliderDisabled>)>,
) {
    let player_pos = player.translation.xy();

    for (entity, transform) in &enemies_enabled {
        if transform.translation.xy().distance(player_pos) > 256.0 {
            commands.entity(entity).insert(ColliderDisabled);
        }
    }

    for (entity, transform) in &enemies_disabled {
        if transform.translation.xy().distance(player_pos) <= 256.0 {
            commands.entity(entity).remove::<ColliderDisabled>();
        }
    }
}

const COLOR_SLEEPING: Color = Color::oklcha(0.668, 0.0, 36.99, 1.00);
const COLOR_AWAKE: Color = Color::oklcha(0.668, 0.224, 36.99, 0.75);

fn enemy_sync_image(
    time: Res<Time<Virtual>>,
    mut enemies: Query<(&mut Sprite, Option<&Awake>, Option<&Awaking>), With<Enemy>>,
) {
    let mut noise = FastNoiseLite::new();
    noise.frequency = 0.1;

    for (mut sprite, awake, awaking) in &mut enemies {
        let color = match (awake, awaking) {
            (Some(awake), _) => {
                let age = time.elapsed_secs() - awake.since.as_secs_f32();
                let amount = (noise.get_noise_2d(awake.seed, age) + 1.0) / 2.0;
                let alpha = amount * 0.3 + 0.5;
                COLOR_AWAKE.with_alpha(alpha)
            }

            (_, Some(awaking)) => {
                let fraction = awaking.timer.fraction();

                COLOR_SLEEPING.mix(&COLOR_AWAKE, fraction.cubed())
            }

            _ => COLOR_SLEEPING,
        };

        if sprite.color != color {
            sprite.color = color;
        }
    }
}

fn state_sleeping(
    mut rand: ResMut<Rand>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &Transform, &Sleeping), With<Sleeping>>,
    time: Res<Time<Virtual>>,
    query_players: Query<&Transform, With<Player>>,
    query_runners: Query<&Transform, (With<Enemy>, With<Awake>)>,
) {
    enum Other<'a> {
        Player(&'a Transform),
        Enemy(&'a Transform),
    }

    impl Other<'_> {
        fn position(&self) -> Vec2 {
            match self {
                Other::Player(tr) => tr.translation.xy(),
                Other::Enemy(tr) => tr.translation.xy(),
            }
        }
    }

    let others: Vec<_> = query_players
        .iter()
        .map(Other::Player)
        .chain(query_runners.iter().map(Other::Enemy))
        .collect();

    for (enemy_id, enemy_transform, enemy_sleeping) in &mut enemies {
        if time.elapsed() - enemy_sleeping.when <= Duration::from_secs(2) {
            // do not wake up again within one second
            continue;
        }

        // get the nearest entity to this one
        let Some(other) = others.iter().min_by_key(|other| {
            OrderedFloat(other.position().distance(enemy_transform.translation.xy()))
        }) else {
            continue;
        };

        // get distance to the player
        let offset = other.position() - enemy_transform.translation.xy();

        let (max_distance, delay_secs_range) = match other {
            Other::Player(_player) => (64.0, 2.0..3.0),
            Other::Enemy(_runner) => (128.0, 0.5..1.0),
        };

        if offset.length() > max_distance {
            // too far away, skipping this one
            continue;
        }

        // wake the guy up and go into the direction of the player
        commands
            .entity(enemy_id)
            .remove::<(Sleeping, ColliderDisabled)>()
            .try_insert((
                Awaking::new(rand.as_mut(), delay_secs_range),
                Squishy {
                    frequency: 1.0,
                    scale_max: Vec2::splat(1.1),
                    scale_min: Vec2::splat(1.0),
                    offset: time.elapsed(),
                },
            ));
    }
}

fn state_awaking(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut rand: ResMut<Rand>,
    mut enemies: Query<(Entity, &mut Transform, &mut Awaking)>,
) {
    for (entity, mut transform, mut awaking) in &mut enemies {
        if !awaking.timer.tick(time.delta()).just_finished() {
            continue;
        }

        transform.scale = Vec3::splat(1.0);

        commands
            .entity(entity)
            .remove::<(Awaking, Squishy, LinearDamping, ColliderDisabled)>()
            .insert((
                Awake {
                    since: time.elapsed(),
                    seed: rand.random_range(0.0..200.0),
                    reorient: Timer::default(),
                },
                Squishy {
                    frequency: rand.random_range(1.8..2.2),
                    scale_min: vec2(0.9, 1.0),
                    scale_max: vec2(1.09, 1.0),
                    offset: Duration::ZERO,
                },
            ));
    }
}

fn state_awake_hunt_player(
    mut rand: ResMut<Rand>,
    time: Res<Time<Virtual>>,
    mut enemies: Query<(&Transform, &mut Awake, &mut LinearVelocity), With<Enemy>>,
    players: Query<&Transform, With<Player>>,
) {
    let players: Vec<_> = players.iter().collect();

    for (enemy_transform, mut enemy_awake, mut enemy_movement) in &mut enemies {
        if !enemy_awake.reorient.tick(time.delta()).just_finished() {
            continue;
        }

        // re-init the timer to reorient later
        enemy_awake.reorient = Timer::new(
            Duration::from_secs_f32(rand.random_range(1.0..2.0)),
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
        let target = player.translation.xy() + rand.vec2() * 32.0;
        let offset = target - enemy_transform.translation.xy();
        enemy_movement.0 = offset.normalize() * rand.random_range(100.0..140.0);
    }
}

fn restrict_number_of_enemies_awake(
    mut commands: Commands,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut ExternalForce,
        ),
        (With<Enemy>, With<Awake>),
    >,
    player: Single<&Transform, With<Player>>,
    time: Res<Time<Virtual>>,
) {
    const MAX: usize = 256;

    // disable enemies that are furthest away from the player, but only if we have more
    // than N active enemies
    let mut enemies: Vec<_> = enemies.iter_mut().collect::<Vec<_>>();
    if enemies.len() < MAX {
        return;
    }

    // sort enemies ascending by
    enemies.sort_by_cached_key(|(_, tr, ..)| {
        OrderedFloat(tr.translation.distance(player.translation))
    });

    for (id, _, mov, angvel, force) in enemies.iter_mut().skip(MAX) {
        mov.0 = Vec2::ZERO;
        angvel.0 = 0.0;
        force.set_force(Vec2::ZERO);

        // revert into sleeping state
        commands.entity(*id).remove::<(Awake, Squishy)>().insert((
            Sleeping {
                when: time.elapsed(),
            },
            ColliderDisabled,
        ));
    }
}

fn state_awake_avoid_collisions(mut enemies: Query<(&mut ExternalForce, &Transform), With<Awake>>) {
    let mut enemies: Vec<_> = enemies.iter_mut().collect();

    for idx in 0..enemies.len() {
        let (_, transform) = enemies[idx];
        let position = transform.translation.xy();

        let mut new_force = Vec2::ZERO;

        // calculate force
        for (other_idx, (_, other)) in enemies.iter().enumerate() {
            if other_idx == idx {
                continue;
            }

            let other_position = other.translation.xy();

            let distance = position.distance(other_position);
            if position.distance(other_position) < 64.0 {
                let direction = (position - other_position).normalize();
                new_force += direction * (1000000.0 / distance).min(1000000.0);
            }
        }

        let (force, _) = &mut enemies[idx];
        force.apply_force(new_force);
    }
}
