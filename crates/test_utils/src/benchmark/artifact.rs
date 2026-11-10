use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const HASH_BUFFER_BYTES: usize = 64 * 1024;

pub struct BenchmarkArtifacts {
    pub directory: PathBuf,
    pub workspace: PathBuf,
}

impl BenchmarkArtifacts {
    pub fn new(owner: &str, profile: &str) -> io::Result<Self> {
        let owner = safe_name(owner)?;
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?;
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();
        let directory = workspace
            .join("target/benchmark")
            .join(format!("{owner}-{time}-{}", std::process::id()));
        fs::create_dir_all(&directory)?;
        let artifacts = Self {
            workspace,
            directory,
        };
        artifacts.write_json(
            "environment.json",
            &json!({
                "measured_at_unix_ms": time,
                "profile": profile,
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "commit": command(&artifacts.workspace, "git", &["rev-parse", "HEAD"]),
                "working_tree": command(&artifacts.workspace, "git", &["status", "--porcelain"]),
                "rust": command(&artifacts.workspace, "rustc", &["-Vv"]),
                "hardware": hardware(),
                "rustflags": std::env::var("RUSTFLAGS").ok(),
            }),
        )?;
        Ok(artifacts)
    }

    pub fn write_json(&self, filename: &str, value: &Value) -> io::Result<PathBuf> {
        let name = safe_name(filename)?;
        let path = self.directory.join(name);
        let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
        fs::write(&path, bytes)?;
        Ok(path)
    }

    pub fn hash_file(path: &Path) -> io::Result<String> {
        let mut file = fs::File::open(path)?;
        let mut digest = Sha256::new();
        let mut buffer = [0u8; HASH_BUFFER_BYTES];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
        }
        Ok(format!("{:x}", digest.finalize()))
    }
}

fn safe_name(name: &str) -> io::Result<&str> {
    if !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && !matches!(name, "." | "..")
    {
        Ok(name)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid artifact name",
        ))
    }
}

fn command(directory: &Path, program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .current_dir(directory)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn hardware() -> Value {
    let Ok(connection) = wmi::WMIConnection::new() else {
        return json!({"available": false});
    };
    let mut result = serde_json::Map::new();
    for (name, query) in [
        (
            "cpu",
            "SELECT Name,NumberOfCores,NumberOfLogicalProcessors FROM Win32_Processor",
        ),
        (
            "gpu",
            "SELECT Name,DriverVersion FROM Win32_VideoController",
        ),
        (
            "os",
            "SELECT Caption,Version,BuildNumber FROM Win32_OperatingSystem",
        ),
    ] {
        let output: Value =
            match connection.raw_query::<std::collections::HashMap<String, Value>>(query) {
                Ok(values) => json!(values),
                Err(error) => json!({"query_error": error.to_string()}),
            };
        result.insert(name.into(), output);
    }
    Value::Object(result)
}

#[cfg(test)]
#[allow(clippy::disallowed_macros, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_in_artifact_names() {
        assert!(safe_name("../outside").is_err());
        assert!(safe_name("results.json").is_ok());
        assert!(safe_name("..").is_err());
    }
}
