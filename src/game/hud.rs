use crate::game::enemy::{Awake, Enemy};
use crate::game::player::Player;
use crate::game::screens::Screen;
use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_event::<AddScore>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn);

    app.add_systems(
        Update,
        (
            update_hud,
            add_score_animation_spawn,
            add_score_animation.after(add_score_animation_spawn),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component)]
enum Hud {
    Score,
    Stats,
}

#[derive(Event)]
pub struct AddScore {
    pub score: u32,
    pub position: Vec2,
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Gameplay),
        Text::new("Score"),
        Hud::Score,
        Node {
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::End,
            margin: UiRect::all(Val::Px(16.0)),
            ..default()
        },
    ));

    commands.spawn((
        StateScoped(Screen::Gameplay),
        Text::new("Stats"),
        Hud::Stats,
        Node {
            align_self: AlignSelf::End,
            justify_self: JustifySelf::End,
            margin: UiRect::all(Val::Px(16.0)),
            ..default()
        },
    ));
}

fn update_hud(
    time: Res<Time<Virtual>>,
    player: Single<&Player>,
    labels: Query<(&mut Text, &Hud)>,
    enemies_awake: Query<(), (With<Enemy>, With<Awake>)>,
) {
    for (mut text, hud) in labels {
        text.set_if_neq(Text::new(match hud {
            Hud::Score => {
                format!("score: {}", player.score(time.elapsed()))
            }

            Hud::Stats => {
                let awake = enemies_awake.iter().count();
                let killed = player.kill_count;
                format!("awake: {}, killed: {}", awake, killed)
            }
        }));
    }
}

#[derive(Component)]
struct AddScoreText {
    lifetime: Timer,
}

fn add_score_animation_spawn(mut commands: Commands, mut events: EventReader<AddScore>) {
    for event in events.read() {
        let position = event.position;

        commands.spawn((
            StateScoped(Screen::Gameplay),
            Text2d::new(format!("+{}", event.score)),
            Transform::from_translation(position.extend(4.0)),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.25)),
            Anchor::BottomCenter,
            AddScoreText {
                lifetime: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
    }
}

fn add_score_animation(
    mut commands: Commands,
    mut texts: Query<(Entity, &mut AddScoreText, &mut Transform, &mut TextColor)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut text, mut transform, mut color) in &mut texts {
        if text.lifetime.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let f = text.lifetime.fraction();
        transform.translation.y += 200.0 * f * time.delta_secs();

        let alpha = text.lifetime.fraction_remaining().squared() * 0.25;
        color.0.set_alpha(alpha);
    }
}
