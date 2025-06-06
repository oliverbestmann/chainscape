use avian2d::prelude::Gravity;
use bevy::prelude::*;

pub mod assets;
pub mod cursor;
pub mod enemy;
mod highscore;
pub mod movement;
pub mod player;
pub mod rand;
pub mod screens;
pub mod squishy;

use crate::game::rand::Rand;
use crate::game::screens::Screen;
pub use assets::Assets;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        cursor::plugin,
        rand::plugin,
        assets::plugin,
        screens::plugin,
        movement::plugin,
        squishy::plugin,
        player::plugin,
        enemy::plugin,
        highscore::plugin,
    ));

    app.add_systems(OnEnter(Screen::Gameplay), spawn_game);

    app.insert_resource(Gravity::ZERO);
}

pub fn spawn_game(mut commands: Commands, mut rand: ResMut<Rand>, assets: Res<Assets>) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Gameplay),
        player::player_bundle(&assets),
    ));

    for pos in enemy::generate_positions(1, Vec2::ZERO, 256.0, 4096.0, 32.0, 4096) {
        let enemy = enemy::Enemy {
            observe_radius: 128.0,
        };

        commands.spawn((
            Name::new("Enemy"),
            StateScoped(Screen::Gameplay),
            enemy::enemy_bundle(rand.as_mut(), &assets, enemy),
            Transform::from_translation(pos.extend(1.0)),
        ));
    }
}
