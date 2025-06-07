use ::rand::seq::IndexedRandom;
use avian2d::prelude::{Gravity, SubstepCount};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub mod assets;
pub mod cursor;
pub mod enemy;
pub mod highscore;
pub mod movement;
pub mod player;
pub mod powerup;
pub mod rand;
pub mod screens;
pub mod squishy;

use crate::game::powerup::{Powerup, powerup_bundle};
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
        powerup::plugin,
    ));

    app.add_systems(OnEnter(Screen::Gameplay), (spawn_game, spawn_background));

    app.insert_resource(Gravity::ZERO);
    app.insert_resource(SubstepCount(3));
}

pub fn spawn_game(mut commands: Commands, mut rand: ResMut<Rand>, assets: Res<Assets>) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Gameplay),
        player::player_bundle(&assets),
    ));

    for pos in enemy::generate_positions(rand.as_mut(), Vec2::ZERO, 256.0, 4096.0, 32.0, 4096) {
        commands.spawn((
            Name::new("Enemy"),
            StateScoped(Screen::Gameplay),
            enemy::enemy_bundle(rand.as_mut(), &assets),
            Transform::from_translation(pos.extend(1.0)),
        ));
    }

    // place some random powerups
    for _ in 0..64 {
        let powerup = [Powerup::Speed, Powerup::Explosion]
            .choose(&mut *rand)
            .unwrap();
        let pos = rand.vec2() * 4096.0;

        commands.spawn((
            Name::new("Powerup"),
            StateScoped(Screen::Gameplay),
            powerup_bundle(&assets, *powerup),
            Transform::from_translation(pos.extend(1.0)),
        ));
    }
}

pub fn spawn_background(
    mut commands: Commands,
    mut images: ResMut<bevy::asset::Assets<Image>>,
    assets: Res<Assets>,
) {
    let image = images.get_mut(&assets.noise).unwrap();
    image.sampler = ImageSampler::nearest();

    commands.spawn((
        Name::new("Background"),
        StateScoped(Screen::Gameplay),
        Transform::from_xyz(0.0, 0.0, -1.0),
        Sprite {
            image: assets.noise.clone(),
            anchor: Anchor::Center,
            custom_size: Some(Vec2::splat(16.0 * 1024.0)),
            ..default()
        },
    ));
}
