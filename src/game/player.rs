use crate::game::cursor::{MainCamera, WorldCursor};
use crate::game::enemy::Enemy;
use crate::game::highscore::ShowHighscore;
use crate::game::movement::Movement;
use crate::game::powerup::{ApplyPowerup, Powerup};
use crate::game::screens::Screen;
use crate::game::squishy::Squishy;
use crate::{AppSystems, PausableSystems, Pause, game};
use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionStarted, LinearVelocity, RigidBody,
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::time::Duration;
use tracing::info;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_player_collision,
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
pub struct Player;

pub fn player_bundle(assets: &game::Assets) -> impl Bundle {
    (
        Player,
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
        CollisionEventsEnabled,
    )
}

fn handle_player_collision(
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    mut events: EventReader<CollisionStarted>,
    query_player: Query<(), With<Player>>,

    query_powerup: Query<&Powerup>,
    query_enemies: Query<(), With<Enemy>>,
) {
    for &CollisionStarted(entity_a, entity_b) in events.read() {
        // sort entities and get the player
        let a_is_player = query_player.contains(entity_a);
        let _player_entity = if a_is_player { entity_a } else { entity_b };
        let collider_entity = if a_is_player { entity_b } else { entity_a };

        // check if we have collided with a powerup
        if let Ok(powerup) = query_powerup.get(collider_entity) {
            info!("Collected powerup {:?}", powerup);
            commands.queue(ApplyPowerup(*powerup));

            // remove the collided entity
            commands.entity(collider_entity).despawn();
        }

        if query_enemies.contains(collider_entity) {
            if let Some(player_name) = player_name() {
                commands.queue(ShowHighscore {
                    player: player_name,
                    score: 1234,
                });
            }

            // pause the systems
            commands.insert_resource(NextState::Pending(Pause(true)));

            // pause time
            time.pause();
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

#[cfg(target_arch = "wasm32")]
fn player_name() -> Option<String> {
    web_sys::window()?
        .get("PlayerName")
        .and_then(|f| f.as_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn player_name() -> Option<String> {
    std::env::var("USER").ok().or_else(|| Some("Test".into()))
}
