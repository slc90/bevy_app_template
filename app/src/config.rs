use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::warn;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

const DEFAULT_SCALE_FACTOR: f32 = 1.25;
const ALLOWED_SCALE_FACTORS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];
const DEFAULT_BRP_PORT: u16 = 15702;
const RENDER_WORLD_BRP_PORT: u16 = 15703;
const BRP_PORT_RANGE: std::ops::RangeInclusive<u16> = 1024..=65534;

#[derive(Resource, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppConfig {
    pub(crate) scale_factor: f32,
    pub(crate) brp_enabled: bool,
    pub(crate) brp_port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            scale_factor: DEFAULT_SCALE_FACTOR,
            brp_enabled: true,
            brp_port: DEFAULT_BRP_PORT,
        }
    }
}

impl AppConfig {
    fn validate(&self) -> io::Result<()> {
        if !ALLOWED_SCALE_FACTORS.contains(&self.scale_factor) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "scale_factor 必须为 1.0、1.25、1.5 或 2.0"));
        }
        if !BRP_PORT_RANGE.contains(&self.brp_port) || self.brp_port == RENDER_WORLD_BRP_PORT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "brp_port 必须为 1024..=65534，且不能占用 15703",
            ));
        }
        Ok(())
    }

    fn serialized(&self) -> io::Result<String> {
        self.validate()?;
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(io::Error::other)
    }

    // Used when the application later provides settings editing. Never overwrite an invalid input during load.
    #[allow(dead_code)]
    pub(crate) fn save(&self, file: &ConfigFile) {
        let result = (|| {
            let content = self.serialized()?;
            file.create_directory()?;
            fs::write(&file.path, content)
        })();
        if let Err(error) = result {
            warn!(path = %file.path.display(), error = %error, "无法保存配置；保留当前内存设置");
        }
    }
}

#[derive(Resource)]
pub(crate) struct ConfigFile {
    path: PathBuf,
}

impl ConfigFile {
    fn create_directory(&self) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "配置文件缺少父目录")
        })?;
        fs::create_dir_all(parent)
    }

    fn create_default(&self, config: &AppConfig) -> io::Result<()> {
        let content = config.serialized()?;
        self.create_directory()?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)?;
        file.write_all(content.as_bytes())
    }
}

pub(crate) struct LoadedConfig {
    pub(crate) config: AppConfig,
    pub(crate) file: ConfigFile,
}

pub(crate) fn load(path: PathBuf) -> LoadedConfig {
    let file = ConfigFile { path };
    let config = match fs::read_to_string(&file.path) {
        Ok(content) => match ron::from_str::<AppConfig>(&content) {
            Ok(config) => match config.validate() {
                Ok(()) => config,
                Err(error) => {
                    warn!(path = %file.path.display(), error = %error, "配置无效；采用默认值并保留原文件");
                    AppConfig::default()
                }
            },
            Err(error) => {
                warn!(path = %file.path.display(), error = %error, "配置解析失败；采用默认值并保留原文件");
                AppConfig::default()
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let config = AppConfig::default();
            if let Err(error) = file.create_default(&config) {
                warn!(path = %file.path.display(), error = %error, "无法创建默认配置；继续使用内存默认值");
            }
            config
        }
        Err(error) => {
            warn!(path = %file.path.display(), error = %error, "配置读取失败；采用默认值并保留原文件");
            AppConfig::default()
        }
    };
    LoadedConfig { config, file }
}

#[cfg(test)]
#[allow(clippy::disallowed_macros, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn malformed_config_is_kept_when_default_is_used() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.ron");
        fs::write(&path, "not ron").unwrap();
        let config = load(path.clone()).config;
        assert_eq!(config.scale_factor, DEFAULT_SCALE_FACTOR);
        assert_eq!(fs::read_to_string(&path).unwrap(), "not ron");
    }

    #[test]
    fn missing_config_is_created_with_defaults() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("nested").join("config.ron");
        let config = load(path.clone()).config;
        assert_eq!(config.brp_port, DEFAULT_BRP_PORT);
        assert_eq!(ron::from_str::<AppConfig>(&fs::read_to_string(path).unwrap()).unwrap().brp_port, DEFAULT_BRP_PORT);
    }

    #[test]
    fn invalid_port_does_not_override_existing_config() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.ron");
        let original = "(scale_factor:1.25,brp_enabled:true,brp_port:15703)";
        fs::write(&path, original).unwrap();
        assert_eq!(load(path.clone()).config.brp_port, DEFAULT_BRP_PORT);
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }
}
