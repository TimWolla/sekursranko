use std::convert::From;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_derive::{Serialize, Deserialize};

/// The server configuration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ServerConfig {
    /// The max file size for backups (e.g. 65536)
    pub max_backup_bytes: u64,
    /// The number of days a backup will be retained (e.g. 180)
    pub retention_days: u32,
    /// The path to the directory where backups will be stored
    pub backup_dir: PathBuf,
    /// The number of threads for doing I/O (e.g. 4)
    pub io_threads: usize,
}

impl ServerConfig {
    pub fn from_file(config_path: &Path) -> Result<Self, String> {
        // Read config file
        if !config_path.exists() {
            return Err(format!("Config file at {:?} does not exist", config_path));
        }
        if !config_path.is_file() {
            return Err(format!("Config file at {:?} is not a file", config_path));
        }
        let mut file = File::open(config_path)
            .map_err(|e| format!("Could not open config file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Could not read config file: {}", e))?;

        // Deserialize
        toml::from_str(&contents)
            .map_err(|e| format!("Could not deserialize config file: {}", e))
    }
}

/// The public part of the server configuration.
///
/// This can be queried over the API.
#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfigPublic {
    /// The max file size for backups (e.g. 65536)
    pub max_backup_bytes: u64,
    /// The number of days a backup will be retained (e.g. 180)
    pub retention_days: u32,
}

impl<'a> From<&'a ServerConfig> for ServerConfigPublic {
    fn from(other: &'a ServerConfig) -> Self {
        Self {
            max_backup_bytes: other.max_backup_bytes,
            retention_days: other.retention_days,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    use tempfile::NamedTempFile;

    #[test]
    fn read_config_file_invalid() {
        let path = Path::new("/tmp/asdfklasdfjaklsdfjlk");
        let res = ServerConfig::from_file(path);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), format!("Config file at {:?} does not exist", path));
    }

    #[test]
    fn read_config_file_no_file() {
        let path = Path::new("/bin");
        let res = ServerConfig::from_file(path);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), format!("Config file at {:?} is not a file", path));
    }

    #[test]
    fn read_config_file_ok() {
        let mut tempfile = NamedTempFile::new().unwrap();
        let file = tempfile.as_file_mut();
        file.write_all(b"max_backup_bytes = 10000\n").unwrap();
        file.write_all(b"retention_days = 100\n").unwrap();
        file.write_all(b"backup_dir = \"backups\"\n").unwrap();
        file.write_all(b"io_threads = 4\n").unwrap();
        let res = ServerConfig::from_file(tempfile.path());
        assert_eq!(res.unwrap(), ServerConfig {
            max_backup_bytes: 10_000,
            retention_days: 100,
            backup_dir: PathBuf::from("backups"),
            io_threads: 4,
        });
    }
}
