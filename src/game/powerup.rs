use crate::game;
use crate::game::squishy::Squishy;
use avian2d::prelude::{Collider, Sensor};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::time::Duration;

pub fn plugin(_app: &mut App) {}

#[derive(Copy, Clone, Component, Debug)]
pub enum Powerup {
    Speed,
    Explosion,
}

pub fn powerup_bundle(assets: &game::Assets, powerup: Powerup) -> impl Bundle {
    (
        powerup,
        Sprite {
            image: match powerup {
                Powerup::Speed => assets.up_speed.clone(),
                Powerup::Explosion => assets.up_explosion.clone(),
            },
            color: Color::oklch(0.868, 0.174, 90.43),
            custom_size: Some(Vec2::splat(32.0)),
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
        Collider::circle(16.0),
    )
}
