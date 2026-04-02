use bevy::prelude::*;

#[derive(Message, Debug, Clone, PartialEq, Reflect)]
pub struct WeatherTransitionStarted {
    pub from_label: Option<String>,
    pub to_label: Option<String>,
    pub duration_secs: f32,
}

#[derive(Message, Debug, Clone, PartialEq, Reflect)]
pub struct WeatherTransitionFinished {
    pub active_label: Option<String>,
}

#[derive(Message, Debug, Clone, PartialEq, Reflect)]
pub struct WeatherProfileChanged {
    pub active_label: Option<String>,
}

#[derive(Message, Debug, Clone, PartialEq, Reflect)]
pub struct LightningFlashEmitted {
    pub flash_id: u64,
    pub intensity: f32,
}
