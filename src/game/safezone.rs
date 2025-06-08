use crate::game;
use crate::game::player::Player;
use crate::game::screens::Screen;
use avian2d::prelude::{Collider, Collisions, Sensor};
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (safezone_reached, safezone_sync_color)
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component)]
pub struct Safezone {
    active: bool,
}

pub fn safezone_bundle(assets: &game::Assets) -> impl Bundle {
    (
        Safezone { active: false },
        Sprite {
            image: assets.safezone.clone(),
            custom_size: Some(Vec2::splat(128.0)),
            anchor: Anchor::Center,
            color: Color::oklch(0.918, 0.238, 127.48),
            ..default()
        },
        Sensor,
        Collider::round_rectangle(64.0, 64.0, 8.0),
    )
}

fn safezone_sync_color(mut query_safezones: Query<(&Safezone, &mut Sprite)>) {
    for (safezone, mut sprite) in &mut query_safezones {
        let alpha = if safezone.active { 1.0 } else { 0.75 };
        sprite.color.set_alpha(alpha);
    }
}

fn safezone_reached(
    collisions: Collisions,
    mut query_safezones: Query<(Entity, &mut Safezone), With<Safezone>>,
    mut query_is_player: Query<&mut Player>,
) {
    for (safezone_entity, mut safezone) in &mut query_safezones {
        if safezone.active {
            safezone.active = false;
        }

        for collider in collisions.entities_colliding_with(safezone_entity) {
            let Ok(mut player) = query_is_player.get_mut(collider) else {
                continue;
            };

            player.safezone_reached = true;
            safezone.active = true;
        }
    }
}
