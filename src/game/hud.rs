use crate::game::enemy::{Awake, Enemy};
use crate::game::player::Player;
use crate::game::screens::Screen;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn);

    app.add_systems(Update, update_hud.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component)]
enum Hud {
    Score,
    Stats,
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
