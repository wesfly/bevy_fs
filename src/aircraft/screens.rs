use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Debug, Component)]
pub enum Screens {
    Left,
    Right,
    Centre,
}

#[derive(Deserialize, Debug)]
pub struct Screen {
    pub screen: Screens,
}
