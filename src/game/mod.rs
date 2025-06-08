use ::rand::seq::IndexedRandom;
use avian2d::prelude::{Collider, DefaultFriction, Friction, Gravity, RigidBody, SubstepCount};
use bevy::ecs::system::RunSystemOnce;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::f32::consts::PI;

pub mod assets;
pub mod cursor;
pub mod enemy;
pub mod highscore;
pub mod movement;
pub mod player;
pub mod powerup;
pub mod rand;
pub mod safezone;
pub mod screens;
pub mod squishy;

use crate::Pause;
use crate::game::highscore::{HighscoreClosed, RecordHighscore};
use crate::game::player::Player;
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
        safezone::plugin,
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
    time: Res<Time<Virtual>>,
    assets: Res<Assets>,
) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Gameplay),
        player::player_bundle(&time, &assets),
        Transform::from_xyz(0.0, 0.0, 0.5),
    ));

    // place the safe zone
    commands.spawn((
        Name::new("SafeZone"),
        StateScoped(Screen::Gameplay),
        safezone::safezone_bundle(&assets),
        Transform::from_xyz(200.0, 100.0, 0.0),
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
    for _ in 0..128 {
        let powerup = [Powerup::Speed, Powerup::Explosion, Powerup::Coin]
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

    if let Some(player_name) = player_name() {
        let score = (time.elapsed() - player.born).as_secs() as u32;
        commands.queue(RecordHighscore {
            player: player_name,
            score,
        });
    }

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
fn player_name() -> Option<String> {
    web_sys::window()?.get("Player").and_then(|f| f.as_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn player_name() -> Option<String> {
    std::env::var("USER").ok().or_else(|| Some("Test".into()))
}
