use bevy::app::{App, Update};
use bevy::math::Vec2;
use bevy::prelude::{Component, Query, Res, Time, Transform};
use bevy::time::Virtual;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Component)]
pub struct Squishy {
    pub offset: Duration,
    pub frequency: f32,
    pub scale_max: Vec2,
    pub scale_min: Vec2,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, apply_squishy);
}

fn apply_squishy(time: Res<Time<Virtual>>, mut squishies: Query<(&mut Transform, &Squishy)>) {
    for (mut transform, squishy) in &mut squishies {
        let time = time.elapsed() - squishy.offset;
        let f = (time.as_secs_f32() * squishy.frequency * 2.0 * PI).sin();

        // scale from -1 .. 1 to 0 .. 1
        let f = (f + 1.0) / 2.0;

        transform.scale = squishy.scale_min.lerp(squishy.scale_max, f).extend(1.0);
    }
}
