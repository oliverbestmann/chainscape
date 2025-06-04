use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn plugin(app: &mut App) {
    app.init_resource::<WorldCursor>();
    app.add_systems(PreUpdate, update_world_cursor);
}

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct WorldCursor(pub Vec2);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

fn update_world_cursor(
    mut coords: ResMut<WorldCursor>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    // There is only one primary window, so we can similarly get it from the query:
    let Ok(window) = q_window.single() else {
        return;
    };

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_position;
    }
}
