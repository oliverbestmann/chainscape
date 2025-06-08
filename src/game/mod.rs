use ::rand::seq::IndexedRandom;
use avian2d::prelude::{Collider, DefaultFriction, Friction, Gravity, RigidBody, SubstepCount};
use bevy::ecs::system::RunSystemOnce;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use fastnoise_lite::FastNoiseLite;
use fastnoise_lite::NoiseType;
use std::f32::consts::PI;

pub mod assets;
pub mod cursor;
pub mod enemy;
pub mod highscore;
mod hud;
mod markers;
pub mod movement;
pub mod player;
pub mod powerup;
pub mod rand;
pub mod safezone;
pub mod screens;
pub mod squishy;

use crate::Pause;
use crate::game::cursor::MainCamera;
use crate::game::highscore::{HighscoreClosed, RecordHighscore};
use crate::game::player::Player;
use crate::game::powerup::{Powerup, powerup_bundle};
use crate::game::rand::{Generate, Rand, weighted_by_noise};
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
        safezone::plugin,
        hud::plugin,
        markers::plugin,
    ));

    app.add_systems(OnEnter(Screen::Reset), reset_to_gameplay);
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_game, spawn_outer_area, spawn_background),
    );

    app.add_systems(
        Update,
        reset_at_highscore_closed_event.run_if(in_state(Screen::Gameplay)),
    );

    app.insert_resource(Gravity::ZERO);
    app.insert_resource(SubstepCount(3));
    app.insert_resource(DefaultFriction(Friction::new(0.0)));
}

fn spawn_game(
    mut commands: Commands,
    mut rand: ResMut<Rand>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
    time: Res<Time<Virtual>>,
    assets: Res<Assets>,
) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Gameplay),
        player::player_bundle(&time, &assets),
        Transform::from_xyz(0.0, 0.0, 0.5),
    ));

    camera.translation.x = 0.0;
    camera.translation.y = 0.0;

    let mut generator = Generate::new(4096.0, 256.0, Vec2::ZERO);

    let random_pos = |radius| rand.vec2() * radius;

    for pos in generator.generate(random_pos, 3, 128.0) {
        // place the safe zone
        commands.spawn((
            Name::new("SafeZone"),
            StateScoped(Screen::Gameplay),
            safezone::safezone_bundle(&assets),
            Transform::from_translation(pos.extend(0.0)),
        ));

        // generate a marker for this safe zone
        commands.spawn((
            Name::new("Marker"),
            StateScoped(Screen::Gameplay),
            markers::bundle(
                &assets,
                markers::Marker {
                    color: safezone::COLOR,
                    target: pos,
                },
            ),
        ));
    }

    // place some random powerups with some space around them
    let random_pos = |radius| rand.vec2() * radius;
    for pos in generator.generate(random_pos, 128, 128.0) {
        let powerup = [Powerup::Speed, Powerup::Explosion, Powerup::Coin]
            .choose(&mut *rand)
            .unwrap();

        commands.spawn((
            Name::new("Powerup"),
            StateScoped(Screen::Gameplay),
            powerup_bundle(&assets, *powerup),
            Transform::from_translation(pos.extend(1.0)),
        ));
    }

    // place zombies based on noise values in chunks
    let mut noise = FastNoiseLite::with_seed(1);
    noise.noise_type = NoiseType::Cellular;
    noise.frequency = 0.001;

    for pos in generator.generate(weighted_by_noise(rand.as_mut(), noise), 4096, 32.0) {
        commands.spawn((
            Name::new("Enemy"),
            StateScoped(Screen::Gameplay),
            enemy::enemy_bundle(rand.as_mut(), &assets),
            Transform::from_translation(pos.extend(1.0)),
        ));
    }
}

fn spawn_outer_area(mut commands: Commands, assets: Res<Assets>) {
    let radius = 4096.0;

    const STEP_SIZE: f32 = 10.0;

    for idx in 0..36 {
        let angle_start = (idx as f32 * STEP_SIZE).to_radians();

        let anchor = Vec2::from_angle(angle_start) * radius;

        let segment_length = radius / (2.0 * PI);

        commands.spawn((
            Name::new("Outer rim"),
            StateScoped(Screen::Gameplay),
            RigidBody::Static,
            Collider::segment(vec2(0.0, -segment_length), vec2(0.0, segment_length)),
            Transform::from_rotation(Quat::from_rotation_z(angle_start))
                .with_translation(anchor.extend(0.0)),
            Sprite {
                image: assets.square.clone(),
                custom_size: Some(vec2(64.0, segment_length * 2.0)),
                anchor: Anchor::CenterLeft,
                color: Color::srgba(0.2, 0.2, 0.2, 1.0),
                ..default()
            },
        ));
    }
}

fn spawn_background(
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

fn reset_at_highscore_closed_event(
    mut events: EventReader<HighscoreClosed>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for _event in events.read() {
        screen.set(Screen::Reset);
    }
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

pub struct EndGame {
    pub win: bool,
}

impl Command for EndGame {
    fn apply(self, world: &mut World) {
        _ = world.run_system_once_with(game_ends_system, self);
    }
}

fn game_ends_system(
    end_game: In<EndGame>,
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    mut query_player: Single<(&Player, &mut Visibility)>,
) {
    let (player, player_visibility) = &mut *query_player;

    commands.queue(RecordHighscore {
        player: player_name(),
        score: player.score(time.elapsed()),
    });

    if !end_game.win {
        // hide the player
        player_visibility.set_if_neq(Visibility::Hidden);
    }

    // pause the systems
    commands.insert_resource(NextState::Pending(Pause(true)));

    // pause time
    time.pause();
}

#[cfg(target_arch = "wasm32")]
fn player_name() -> String {
    let Some(window) = web_sys::window() else {
        return "Unknown".into();
    };

    window
        .get("Player")
        .and_then(|f| f.as_string())
        .filter(|name| name.chars().any(|ch| !ch.is_whitespace()))
        .unwrap_or_else(|| String::from("Unknown"))
}

#[cfg(not(target_arch = "wasm32"))]
fn player_name() -> String {
    std::env::var("USER")
        .ok()
        .filter(|name| name.chars().any(|ch| !ch.is_whitespace()))
        .unwrap_or_else(|| String::from("Test"))
}
