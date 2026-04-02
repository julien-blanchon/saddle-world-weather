use bevy::{asset::AssetPlugin, prelude::*, transform::TransformPlugin};
use weather::{WeatherPlugin, WeatherSystems};

#[test]
fn public_plugin_and_sets_are_usable() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        WeatherPlugin::always_on(Update),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Image>>();
    app.add_systems(Update, (|| {}).after(WeatherSystems::Diagnostics));
    app.update();

    assert!(app.world().contains_resource::<weather::WeatherRuntime>());
}
