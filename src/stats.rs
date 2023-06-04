use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::assets;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_section = move |color, value: &str| {
        TextSection::new(
            value,
            TextStyle {
                font: asset_server.load(assets::DEFAULT_FONT_PATH),
                font_size: 20.0,
                color,
            },
        )
    };

    commands.spawn((
        TextBundle::from_sections([
            text_section(Color::GREEN, "\nFPS (raw): "),
            text_section(Color::CYAN, ""),
            text_section(Color::CYAN, ""),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        FpsCounter,
    ));
}

#[derive(Component)]
struct FpsCounter;

fn fps_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsCounter>>) {
    let mut text = query.single_mut();

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(avg) = fps.average() {
            text.sections[1].value = format!("{avg:.2}");
        }
    };
}

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(fps_system);
    }
}
