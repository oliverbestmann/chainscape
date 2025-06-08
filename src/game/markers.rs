use crate::game;
use crate::game::player::Player;
use crate::game::screens::Screen;
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_markers.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component)]
pub struct Marker {
    pub target: Vec2,
    pub color: Color,
}

pub fn bundle(assets: &game::Assets, marker: Marker) -> impl Bundle {
    (
        Sprite {
            image: assets.arrow.clone(),
            custom_size: Some(Vec2::splat(32.0)),
            color: marker.color,
            anchor: Anchor::Center,
            ..default()
        },
        marker,
    )
}

fn update_markers(
    player: Single<&Transform, (With<Player>, Without<Marker>)>,
    mut markers: Query<(&mut Transform, &mut Sprite, &Marker)>,
) {
    for (mut marker_transform, mut marker_sprite, marker) in markers.iter_mut() {
        let direction = marker.target - player.translation.xy();
        let position = player.translation + (direction.normalize() * 24.0).extend(0.);
        let rotation = direction.to_angle();

        marker_transform.translation = position;
        marker_transform.rotation = Quat::from_rotation_z(rotation);

        let alpha = 0.25 + 0.75 * (1.0 - direction.length().min(2048.0) / 2048.0);
        marker_sprite.color = marker.color.with_alpha(alpha);
    }
}
