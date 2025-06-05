use crate::{AppSystems, PausableSystems};
use avian2d::prelude::LinearVelocity;
use bevy::app::{App, Update};
use bevy::math::{vec3, Quat, Vec2, Vec3Swizzles};
use bevy::prelude::{Component, IntoScheduleConfigs, Query, Res, Transform};
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
    timer: Res<Time<Virtual>>,
    mut entities: Query<(&mut Transform, &Movement, &mut LinearVelocity)>,
) {
    let dt = timer.delta_secs();

    for (mut transform, mov, mut velocity) in &mut entities {
        let target_angle = mov.target_velocity.to_angle();
        let target_quat = Quat::from_rotation_z(target_angle);

        // rotate towards the target
        transform.rotation = transform
            .rotation
            .rotate_towards(target_quat, mov.angular_velocity * dt);

        // update velocity
        let current_velocity = transform.rotation * vec3(1.0, 0.0, 0.0);
        velocity.0 = current_velocity.xy().normalize() * mov.target_velocity.length();
    }
}
