use bevy::input::{ButtonState, keyboard::{Key, KeyboardInput, NativeKey}};
use bevy::input_focus::InputFocusPlugin;
use bevy::prelude::*;
use bevy::window::Ime;
use std::time::{Duration, Instant};

const DEFAULT_VIEWPORT: UVec2 = UVec2::splat(600);

pub fn scene_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        InputFocusPlugin,
        bevy::text::TextPlugin,
    ));
    app.set_default_font(bevy::text::FontSource::monospace());
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>();
    app.add_plugins(bevy::camera::visibility::VisibilityPlugin);
    app.init_resource::<bevy::picking::pointer::PointerMap>();
    app.init_resource::<bevy::input::mouse::MouseScrollPixelsPerLine>();
    app.init_asset::<bevy::scene::ScenePatch>();
    app.add_message::<bevy::window::WindowCloseRequested>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app
}

pub fn add_ui_plugins(app: &mut App) {
    if !app.is_plugin_added::<bevy::text::TextPlugin>() {
        app.add_plugins(bevy::text::TextPlugin);
    }
    app.init_resource::<bevy::text::FontCx>()
        .init_resource::<bevy::text::ScaleCx>()
        .init_resource::<bevy::text::TextPipeline>()
        .init_resource::<bevy::input::touch::Touches>()
        .add_message::<bevy::window::WindowEvent>()
        .init_asset::<bevy::image::TextureAtlasLayout>()
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(bevy::picking::DefaultPickingPlugins)
        .add_plugins(bevy::ui::UiPlugin);
}

pub fn text_input_app() -> App {
    let mut app = scene_app();
    app.init_resource::<ButtonInput<Key>>()
        .init_resource::<UiScale>()
        .add_message::<Ime>()
        .add_message::<bevy::picking::events::PointerRelease>()
        .init_resource::<bevy::picking::events::PointerState>()
        .add_plugins(bevy::ui_widgets::TextInputPlugin);
    app
}

pub fn text_edit_app() -> App {
    let mut app = text_input_app();
    add_ui_plugins(&mut app);
    spawn_ui_camera(&mut app, DEFAULT_VIEWPORT, 1.0);
    app
}

pub fn spawn_ui_camera(app: &mut App, physical_size: UVec2, scale_factor: f32) -> Entity {
    app.world_mut()
        .spawn((
            Camera2d,
            Camera {
                computed: bevy::camera::ComputedCameraValues {
                    target_info: Some(bevy::camera::RenderTargetInfo {
                        physical_size,
                        scale_factor,
                    }),
                    ..default()
                },
                viewport: Some(bevy::camera::Viewport {
                    physical_size,
                    ..default()
                }),
                ..default()
            },
        ))
        .id()
}

pub fn press_key(app: &mut App, window: Entity, key_code: KeyCode) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
}

pub fn advance_until(
    app: &mut App,
    timeout: Duration,
    description: &str,
    mut ready: impl FnMut(&World) -> bool,
) -> Result<usize, String> {
    let start = Instant::now();
    let mut frames = 0;
    while !ready(app.world()) {
        if start.elapsed() >= timeout {
            return Err(format!("Timed out waiting for {description} after {frames} frames"));
        }
        app.update();
        frames += 1;
        std::thread::yield_now();
    }
    Ok(frames)
}
