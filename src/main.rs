// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowCreated, WindowResolution};
use bevy::winit::WinitWindows;
use inhabitants::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Inhabitants".into(),
                resolution: WindowResolution::new(1500, 900),
                present_mode: PresentMode::AutoVsync,
                resize_constraints: WindowResizeConstraints {
                    min_width: 800.0,
                    min_height: 600.0,
                    // currently loading (from a save file) not working if
                    // these are not specified explicitly
                    max_width: 100000.0,
                    max_height: 100000.0,
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}

fn set_window_icon(
    windows: Option<NonSend<WinitWindows>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let Some(windows) = windows else { return; };
    let primary_entity = primary_window.single().unwrap();
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!("../assets/icons/icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

// fn set_window_icon_on_create(
//     mut created: MessageReader<WindowCreated>,
//     windows: Option<NonSend<WinitWindows>>,
// ) {
//     let Some(windows) = windows else { return; };
//     panic!("daf");
//     for msg in created.read() {
//         if let Some(win) = windows.get_window(msg.window) {
//             panic!("daf");
//             let icon_buf = Cursor::new(include_bytes!("../assets/icons/icon.png"));
//             if let Ok(img) = image::load(icon_buf, image::ImageFormat::Png) {
//                 let img = img.into_rgba8();
//                 let (w_px, h_px) = img.dimensions();
//                 if let Ok(icon) = Icon::from_rgba(img.into_raw(), w_px, h_px) {
//                     win.set_window_icon(Some(icon));
//                 }
//             }
//         }
//     }
// }
