use crate::game::EndGame;
use crate::game::player::Player;
use crate::game::screens::Screen;
use crate::{PausableSystems, game};
use avian2d::prelude::{Collider, Collisions, Sensor};
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (safezone_reached, safezone_sync_color)
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

pub const COLOR: Color = Color::oklch(0.918, 0.238, 127.48);

#[derive(Component)]
pub struct Safezone;
pub fn safezone_bundle(assets: &game::Assets) -> impl Bundle {
    (
        Safezone,
        Sprite {
            image: assets.safezone.clone(),
            custom_size: Some(Vec2::splat(128.0)),
            anchor: Anchor::Center,
            color: COLOR,
            ..default()
        },
        Sensor,
        Collider::round_rectangle(48.0, 48.0, 8.0),
    )
}

fn safezone_sync_color(
    player: Single<&Transform, With<Player>>,
    mut safezones: Query<(&Transform, &mut Sprite), With<Safezone>>,
) {
    for (sz_transform, mut sprite) in &mut safezones {
        let distance = sz_transform
            .translation
            .xy()
            .distance(player.translation.xy());

        let alpha = 0.25 + 0.75 * (1.0 - distance.min(256.0) / 256.0);
        if sprite.color.alpha() != alpha {
            sprite.color.set_alpha(alpha);
        }
    }
}

fn safezone_reached(
    mut commands: Commands,
    collisions: Collisions,
    query_safezones: Query<Entity, With<Safezone>>,
    mut query_player: Query<&mut Player>,
) {
    for safezone_entity in &query_safezones {
        for collider in collisions.entities_colliding_with(safezone_entity) {
            let Ok(mut player) = query_player.get_mut(collider) else {
                continue;
            };

            // give player an extra bonus for reaching the safezone
            player.safezone_reached = true;

            commands.queue(EndGame { win: true });
        }
    }
}
