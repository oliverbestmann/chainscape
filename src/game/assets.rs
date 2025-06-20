use crate::asset_tracking::LoadResource;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.load_resource::<Assets>();
}

#[derive(Clone, Resource, Asset, TypePath)]
pub struct Assets {
    pub player: Handle<Image>,
    pub enemy: Handle<Image>,
    pub up_speed: Handle<Image>,
    pub up_explosion: Handle<Image>,
    pub up_coin: Handle<Image>,
    pub noise: Handle<Image>,
    pub circle: Handle<Image>,
    pub square: Handle<Image>,
    pub safezone: Handle<Image>,
    pub arrow: Handle<Image>,
}

impl FromWorld for Assets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource_mut::<AssetServer>();

        Self {
            player: server.load("images/player.png"),
            enemy: server.load("images/enemy.png"),
            up_speed: server.load("images/speed.png"),
            up_explosion: server.load("images/explosion.png"),
            up_coin: server.load("images/coin.png"),
            noise: server.load("images/noise.png"),
            circle: server.load("images/circle.png"),
            square: server.load("images/square.png"),
            safezone: server.load("images/safezone.png"),
            arrow: server.load("images/arrow.png"),
        }
    }
}
