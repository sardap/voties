use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    SettingUpWorld,
    SettingUpUi,
    Running,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LifeSet {
    World,
    Decide,
    Goal,
    Mortal,
    MortalResponse,
}
