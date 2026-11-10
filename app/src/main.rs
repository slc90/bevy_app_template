#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(all(target_os = "windows", target_pointer_width = "64")))]
compile_error!("This application template supports Windows x64 only");

mod assets;
mod config;
mod diagnostics;

use assets::AppAssetPlugin;
use bevy::log::LogPlugin;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowPosition, WindowResolution};
use bevy::winit::WinitSettings;
use bevy::{prelude::*, render::RenderPlugin, tasks::block_on};
use bevy_brp_runtime::BrpRuntimePlugin;
use bevy_widgetry::app_logging::{AppLogging, LoggingGuard, log_plugin};
use bevy_widgetry::scene::WidgetrySceneCommandsExt;
use bevy_widgetry::ui_scale::{WidgetryUiScale, WidgetryUiScalePlugin};
use tracing::debug;
use bevy_widgetry::window::{
    WidgetryWindowBackground, WidgetryWindowControlsConfig, WidgetryWindowPlugin,
    prepare_native_window, transparent_render_creation, widgetry_window,
};

fn main() -> std::process::ExitCode {
    let mut logging_ready = false;
    let mut guard: Option<LoggingGuard> = None;
    let result = run(&mut logging_ready, &mut guard);
    let mut exit = match result {
        Ok(exit) => exit,
        Err(error) => {
            diagnostics::startup(error, logging_ready);
            AppExit::error()
        }
    };
    if let Some(guard) = guard
        && let Err(failure) = guard.finish()
    {
        diagnostics::logging_shutdown(&failure);
        exit = AppExit::error();
    }
    match exit {
        AppExit::Success => std::process::ExitCode::SUCCESS,
        AppExit::Error(code) => std::process::ExitCode::from(code.get()),
    }
}

fn run(logging_ready: &mut bool, guard: &mut Option<LoggingGuard>) -> Result<AppExit> {
    let executable = std::env::current_exe().map_err(diagnostics::operation)?;
    let logging = AppLogging::new(executable.with_file_name("logs"))
        .map_err(|failure| failure.into_bevy_error())?;
    let mut app = App::new();
    app.set_error_handler(diagnostics::handler);
    *guard = Some(logging.install(&mut app));
    app.add_plugins(log_plugin());
    *logging_ready = true;

    let loaded = config::load(executable.with_file_name("config").join("config.ron"));
    app.insert_resource(
        WidgetryUiScale::new(loaded.config.scale_factor)
            .map_err(|failure| failure.into_bevy_error())?,
    )
    .add_plugins(WidgetryUiScalePlugin);
    app.insert_resource(WinitSettings::desktop_app());

    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<LogPlugin>()
            .set(WindowPlugin {
                primary_window: Some(initial_window()),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: block_on(transparent_render_creation())
                    .map_err(|failure| failure.into_bevy_error())?,
                ..default()
            }),
    );
    if loaded.config.brp_enabled {
        app.add_plugins(BrpRuntimePlugin::with_port(loaded.config.brp_port));
    }
    app.insert_resource(loaded.config)
        .insert_resource(loaded.file)
        .add_plugins((AppAssetPlugin, WidgetryWindowPlugin))
        .add_systems(Startup, setup);
    debug!("应用装配完成，进入主循环");
    Ok(app.run())
}

fn initial_window() -> Window {
    prepare_native_window(Window {
        title: "app".into(),
        resolution: WindowResolution::new(1920, 1080),
        position: WindowPosition::Centered(MonitorSelection::Primary),
        ..default()
    })
}

fn setup(
    mut commands: Commands,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) -> Result {
    let target = primary_window.single().map_err(diagnostics::operation)?;
    let camera = commands.spawn(Camera2d).id();
    commands.spawn_scene_with_error_handler(bsn! {
        @widgetry_window(
            target,
            camera,
            WidgetryWindowControlsConfig::default(),
            WidgetryWindowBackground::Theme,
            Default::default(),
            bsn_list! {
                bevy_widgetry::window::WidgetryWindowTitle
                bevy_widgetry::text::WidgetryText
                template(|_| Ok(Pickable::IGNORE))
            },
            bsn_list! {
                Node { width: percent(100), height: percent(100) }
            }
        )
    });
    Ok(())
}
