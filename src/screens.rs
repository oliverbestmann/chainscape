use bevy::app::App;

mod loading;

pub fn plugin(app: &mut App) {
    app.add_plugins((loading::plugin,));
}
