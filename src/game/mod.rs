use bevy::prelude::*;

pub mod assets;
pub mod cursor;
pub mod enemy;
pub mod movement;
pub mod player;
pub mod screens;

use crate::game::screens::Screen;
pub use assets::Assets;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        cursor::plugin,
        assets::plugin,
        screens::plugin,
        movement::plugin,
        player::plugin,
        enemy::plugin,
    ));

    app.add_systems(OnEnter(Screen::Gameplay), spawn_game);
}

pub fn spawn_game(mut commands: Commands, assets: Res<Assets>) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Gameplay),
        player::player_bundle(&assets),
    ));

    for pos in enemy::generate_positions(1, Vec2::ZERO, 128.0, 1024.0, 32.0, 100) {
        let enemy = enemy::Enemy {
            observe_radius: 128.0,
        };

        commands.spawn((
            Name::new("Enemy"),
            StateScoped(Screen::Gameplay),
            enemy::enemy_bundle(&assets, enemy),
            Transform::from_translation(pos.extend(1.0)),
        ));
    }
}
