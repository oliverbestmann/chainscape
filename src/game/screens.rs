use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Loading,
    Gameplay,
}