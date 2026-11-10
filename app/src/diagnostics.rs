use bevy::ecs::error::{ErrorContext, Severity};
use bevy::prelude::*;
use bevy_widgetry::diagnostics::{ErrorFrame, WidgetryFailure};
use tracing::{debug, error, info, warn};
use std::{error::Error, fmt};

type Source = Box<dyn Error + Send + Sync>;
type AppFailure = WidgetryFailure<AppError>;

#[derive(Debug)]
pub(crate) enum AppError {
    Operation(Source),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operation(_) => f.write_str("Application operation failed"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Operation(source) => Some(source.as_ref()),
        }
    }
}

#[track_caller]
fn failure(source: impl Into<Source>) -> AppFailure {
    WidgetryFailure::new(AppError::Operation(source.into()), ErrorFrame::new("app.operation"))
}

#[track_caller]
pub(crate) fn operation(source: impl Into<Source>) -> BevyError {
    failure(source).into_bevy_error()
}

pub(crate) fn handler(error: BevyError, context: ErrorContext) {
    match error.severity() {
        Severity::Ignore => {}
        Severity::Trace | Severity::Debug => {
            debug!(error = %error, details = ?error, ecs_context = ?context, "ECS 操作未能完成")
        }
        Severity::Info => {
            info!(error = %error, details = ?error, ecs_context = ?context, "ECS 操作未能完成")
        }
        Severity::Warning => {
            warn!(error = %error, details = ?error, ecs_context = ?context, "ECS 操作未能完成")
        }
        Severity::Error => {
            error!(error = %error, details = ?error, ecs_context = ?context, "ECS 操作失败")
        }
        Severity::Panic => bevy::ecs::error::panic(error, context),
    }
}

pub(crate) fn startup(error: BevyError, logging_ready: bool) {
    if logging_ready {
        error!(error = %error, details = ?error, "应用启动失败");
    } else {
        eprintln!("应用启动失败：{error:?}");
    }
}

pub(crate) fn logging_shutdown(failure: &bevy_widgetry::app_logging::LoggingFailure) {
    eprintln!("日志收尾失败：{failure:?}");
}
