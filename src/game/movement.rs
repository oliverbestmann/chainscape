use crate::{AppSystems, PausableSystems};
use avian2d::prelude::LinearVelocity;
use bevy::app::{App, Update};
use bevy::math::{Quat, Vec2, Vec3Swizzles, vec3};
use bevy::prelude::{Component, DetectChangesMut, IntoScheduleConfigs, Query, Res, Transform};
use bevy::time::{Time, Virtual};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        apply_movement
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct Movement {
    pub target_velocity: Vec2,
    pub angular_velocity: f32,
}

fn apply_movement(
    time: Res<Time<Virtual>>,
    mut entities: Query<(&mut Transform, &Movement, &mut LinearVelocity)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mov, mut velocity) in &mut entities {
        let target_angle = mov.target_velocity.to_angle();
        let target_quat = Quat::from_rotation_z(target_angle);

        // rotate towards the target
        transform.rotation = transform
            .rotation
            .rotate_towards(target_quat, mov.angular_velocity * dt);

        let direction = transform.rotation * vec3(1.0, 0.0, 0.0);
        let current_velocity = direction.xy() * mov.target_velocity.length();

        velocity.set_if_neq(LinearVelocity(current_velocity));
    }
}
