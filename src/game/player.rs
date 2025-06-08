use crate::game::cursor::{MainCamera, WorldCursor};
use crate::game::enemy::{Awake, Enemy};
use crate::game::movement::Movement;
use crate::game::screens::Screen;
use crate::game::squishy::Squishy;
use crate::game::EndGame;
use crate::{game, AppSystems, PausableSystems, Pause};
use avian2d::prelude::{Collider, Collisions, LinearVelocity, RigidBody};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::time::Duration;
use tracing::info;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_player_enemy_collision_awake,
            handle_player_enemy_collision_non_awake,
            handle_player_input,
            handle_player_input_touch,
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    app.add_systems(
        PostUpdate,
        camera_follow_player
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Component)]
pub struct Player {
    pub born: Duration,
    pub bonus_score: u32,
    pub kill_count: u32,
    pub safezone_reached: bool,
}

impl Player {
    pub fn score(&self, now: Duration) -> u32 {
        let age = (now - self.born).as_secs() as u32;
        let safezone = if self.safezone_reached { 100 } else { 0 };
        age + self.bonus_score + self.kill_count * 10
    }
}

pub fn player_bundle(time: &Time<Virtual>, assets: &game::Assets) -> impl Bundle {
    (
        Player {
            born: time.elapsed(),
            bonus_score: 0,
            kill_count: 0,
            safezone_reached: false,
        },
        Movement {
            target_velocity: Vec2::ZERO,
            angular_velocity: 8.0,
        },
        Squishy {
            frequency: 2.0,
            scale_min: vec2(1.0, 0.9),
            scale_max: vec2(1.0, 1.1),
            offset: Duration::ZERO,
        },
        Sprite {
            image: assets.player.clone(),
            custom_size: Some(Vec2::splat(32.0)),
            color: Color::oklch(0.645, 0.260, 2.47),
            anchor: Anchor::Center,
            ..default()
        },
        RigidBody::Dynamic,
        Collider::circle(16.0),
        LinearVelocity::ZERO,
    )
}

fn handle_player_enemy_collision_awake(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    query_enemies: Query<(), (With<Enemy>, With<Awake>)>,
    collisions: Collisions,
) {
    for collider in collisions.entities_colliding_with(*player) {
        if query_enemies.contains(collider) {
            commands.queue(EndGame { win: false });
            return;
        }
    }
}

fn handle_player_enemy_collision_non_awake(
    mut commands: Commands,
    mut query_player: Single<(Entity, &mut Player)>,
    query_enemies: Query<(), (With<Enemy>, Without<Awake>)>,
    collisions: Collisions,
) {
    let (player_entity, player) = &mut *query_player;

    for collider in collisions.entities_colliding_with(*player_entity) {
        if query_enemies.contains(collider) {
            // kill that enemy
            player.kill_count += 1;

            //  and remove it from the map
            commands.entity(collider).despawn();
        }
    }
}

fn handle_player_input(
    cursor: Res<WorldCursor>,
    mut unpause: ResMut<NextState<Pause>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut query_player: Query<(&Transform, &mut Movement), With<Player>>,
) {
    let Ok((player_transform, mut player_movement)) = query_player.single_mut() else {
        return;
    };

    if mouse_button.just_pressed(MouseButton::Left) {
        info!("Mouse button was just pressed at {:?}", cursor.0);

        let player_pos = player_transform.translation.xy();
        let target_pos = cursor.0;

        // direction the player wants to move to
        let direction = target_pos - player_pos;

        // clicked on the player itself
        if direction.length_squared() < 0.01 {
            return;
        }

        // turn around and move!
        player_movement.target_velocity = 130.0 * direction.normalize();

        unpause.set(Pause(false));
    }
}

fn handle_player_input_touch(
    touches: Res<Touches>,
    mut unpause: ResMut<NextState<Pause>>,
    mut query_player: Query<(&Transform, &mut Movement), With<Player>>,

    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((player_transform, mut player_movement)) = query_player.single_mut() else {
        return;
    };

    if touches.any_just_pressed() {
        let Some(pos) = touches.first_pressed_position() else {
            return;
        };

        // get the camera info and transform
        // assuming there is exactly one main camera entity, so Query::single() is OK
        let Ok((camera, camera_transform)) = q_camera.single() else {
            return;
        };

        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        let Some(world_position) = camera
            .viewport_to_world(camera_transform, pos)
            .ok()
            .map(|ray| ray.origin.truncate())
        else {
            return;
        };

        info!(
            "Touch input at {:?}, world position {:?}",
            pos, world_position
        );

        let player_pos = player_transform.translation.xy();
        let target_pos = world_position;

        // direction the player wants to move to
        let direction = target_pos - player_pos;

        // clicked on the player itself
        if direction.length_squared() < 0.01 {
            return;
        }

        // turn around and move!
        player_movement.target_velocity = 130.0 * direction.normalize();

        unpause.set(Pause(false));
    }
}

fn camera_follow_player(
    time: Res<Time<Virtual>>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
    players: Query<(&Transform, &LinearVelocity), (With<Player>, Without<MainCamera>)>,
) {
    let Some((player_transform, player_velocity)) = players.iter().next() else {
        return;
    };

    // the target we want to reach within a short while
    let target = player_transform.translation.xy() + player_velocity.0;

    // current position
    let mut current = camera.translation.xy();

    // update current to go to target
    current.smooth_nudge(&target, 2.0, time.delta_secs());

    // nudge the current camera position into the direction of the target
    camera.translation.x = current.x;
    camera.translation.y = current.y;
}
