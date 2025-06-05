use crate::game::cursor::{MainCamera, WorldCursor};
use crate::game::movement::Movement;
use crate::game::squishy::Squishy;
use crate::{AppSystems, Pause, game};
use avian2d::prelude::{Collider, LinearVelocity, RigidBody};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::time::Duration;
use tracing::info;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_player_input,
            handle_player_input_touch,
            camera_follow,
        )
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
            color: Color::srgba_u8(220, 105, 190, 255),
            anchor: Anchor::Center,
            ..default()
        },
        RigidBody::Kinematic,
        Collider::circle(16.0),
        LinearVelocity::ZERO,
    )
}

pub fn handle_player_input(
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

pub fn handle_player_input_touch(
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

pub fn camera_follow(
    time: Res<Time<Virtual>>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
    players: Query<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };

    let mut target = camera.translation.xy();

    // after one second distance is 1/10th
    let decay_rate = f32::ln(10.0);

    target.smooth_nudge(&player.translation.xy(), decay_rate, time.delta_secs());
    camera.translation.x = target.x;
    camera.translation.y = target.y;
}
