use crate::asset_tracking::LoadResource;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.load_resource::<Assets>();
}

#[derive(Clone, Resource, Asset, TypePath)]
pub struct Assets {
    pub player: Handle<Image>,
    pub enemy: Handle<Image>,
    pub radius: Handle<Image>,
}

impl FromWorld for Assets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource_mut::<AssetServer>();

        let player = server.load("images/player.png");
        let enemy = server.load("images/enemy.png");
        let radius = server.load("images/radius.png");

        Self {
            player,
            enemy,
            radius,
        }
    }
}
