use crate::game;
use crate::game::screens::Screen;
use bevy::app::{App, Update};
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, IoTaskPool, Task};
use bevy::ui::{Node, Val};
use serde::Deserialize;
use tracing::info;

pub fn plugin(app: &mut App) {
    app.init_resource::<Highscore>();

    app.add_systems(
        Update,
        display_if_available.run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Debug, Deserialize)]
pub struct HighscoreItem {
    pub player: String,
    pub score: usize,
}

pub type Response = Result<Vec<HighscoreItem>, String>;

#[derive(Default, Resource)]
pub struct Highscore {
    // the currently running task that fetches the highscore.
    task: Option<Task<Response>>,
}

impl Highscore {
    pub fn take(&mut self) -> Option<Response> {
        if let Some(mut task) = self.task.as_mut() {
            if let Some(resp) = block_on(future::poll_once(&mut task)) {
                // clear the task
                self.task = None;

                // return the response
                return Some(resp);
            }
        }

        None
    }

    pub fn post(&mut self, player: &str, score: u32) {
        if let Some(task) = self.task.take() {
            // cancel the previous task
            let _ = task.cancel();
        }

        info!("Reporting highscore {} for player {:?}", score, player);

        let url = url::Url::parse_with_params(
            "https://highscore.narf.zone/games/chainscape-1/highscore",
            &[("player", player), ("score", &score.to_string())],
        );

        // create the request
        let req = ehttp::Request::post(url.unwrap(), Vec::new());

        // and schedule it to be processed asynchronously
        let task = IoTaskPool::get().spawn(async move {
            let resp = ehttp::fetch_async(req).await;

            match resp {
                Ok(resp) if resp.ok => {
                    info!("Got successful response, parsing highscore now");
                    match serde_json::from_slice::<Vec<HighscoreItem>>(&resp.bytes) {
                        Err(err) => Err(format!("Failed to parse highscore response: {:?}", err)),

                        Ok(highscore) => {
                            info!("Highscore contains {} items", highscore.len());
                            Ok(highscore)
                        }
                    }
                }

                Ok(resp) => Err(format!(
                    "Failed to report highscore, got status code {:?}",
                    resp.status
                )),

                Err(err) => Err(format!("Failed to report highscore: {:?}", err)),
            }
        });

        self.task = Some(task);
    }
}

fn display_if_available(
    mut commands: Commands,
    mut highscore: ResMut<Highscore>,
    _handles: Res<game::Assets>,
) {
    let Some(mut highscore) = highscore.take() else {
        return;
    };

    if let Ok(highscore) = &mut highscore {
        // sort by key descending
        highscore.sort_by_key(|h| h.score);
        highscore.reverse();
    }

    commands
        .spawn((Node {
            width: Val::Percent(80.0),
            max_width: Val::Px(480.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                // the title
                Text::new("Highscore"),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..Default::default()
                },
            ));

            if let Ok(entries) = &highscore {
                for entry in entries.iter().take(20) {
                    parent.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    });

                    parent.spawn((
                        Text::new(&entry.player),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));

                    parent.spawn((Text::new(&entry.score.to_string()),));
                }
            }
        });
}
