use avian2d::prelude::{Gravity, SubstepCount};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use ::rand::seq::IndexedRandom;

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

use crate::game::highscore::HighscoreClosed;
use crate::game::powerup::{powerup_bundle, Powerup};
use crate::game::rand::Rand;
use crate::game::screens::Screen;
use crate::Pause;
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

    app.add_systems(OnEnter(Screen::Reset), reset_to_gameplay);
    app.add_systems(OnEnter(Screen::Gameplay), (spawn_game, spawn_background));

    app.add_systems(
        Update,
        reset_at_highscore_closed_event.run_if(in_state(Screen::Gameplay)),
    );

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
    if let Some(image) = images.get_mut(&assets.noise) {
        image.sampler = ImageSampler::nearest();
    }

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

fn reset_to_gameplay(
    mut time: ResMut<Time<Virtual>>,
    mut pause: ResMut<NextState<Pause>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    time.unpause();
    pause.set(Pause(false));
    screen.set(Screen::Gameplay);
}

fn reset_at_highscore_closed_event(
    mut events: EventReader<HighscoreClosed>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for _event in events.read() {
        screen.set(Screen::Reset);
    }
}
