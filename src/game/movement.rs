use crate::{AppSystems, PausableSystems};
use bevy::app::{App, Update};
use bevy::math::{Quat, Vec2, vec3};
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
    pub velocity: Vec2,
}

fn apply_movement(timer: Res<Time<Virtual>>, mut entities: Query<(&mut Transform, &Movement)>) {
    let dt = timer.delta_secs();

    for (mut transform, mov) in &mut entities {
        if mov.velocity.length_squared() < 0.01 {
            continue;
        }

        let target_angle = mov.velocity.to_angle();
        let target_quat = Quat::from_rotation_z(target_angle);

        // rotate towards the target
        transform.rotation = transform.rotation.rotate_towards(target_quat, 16.0 * dt);

        // get the direction we're looking at right now and move into that direction
        let velocity = transform.rotation * vec3(1.0, 0.0, 0.0) * mov.velocity.length();
        transform.translation += velocity * dt;
    }
}
