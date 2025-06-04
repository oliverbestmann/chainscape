use crate::game::cursor::WorldCursor;
use crate::game::movement::Movement;
use crate::{game, AppSystems, PausableSystems, Pause};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use tracing::info;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_player_input.in_set(AppSystems::Update));
}

#[derive(Component)]
pub struct Player;

pub fn player_bundle(assets: &game::Assets) -> impl Bundle {
    (
        Player,
        Movement {
            velocity: Vec2::ZERO,
        },
        Sprite {
            image: assets.player.clone(),
            custom_size: Some(Vec2::splat(32.0)),
            color: Color::srgba_u8(220, 105, 190, 255),
            anchor: Anchor::Center,
            ..default()
        },
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
        player_movement.velocity = 100.0 * direction.normalize();

        unpause.set(Pause(false));
    }
}
