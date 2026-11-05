use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn digest(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
pub fn read<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match fs::read(path) {
        Ok(bytes) => {
            Ok(Some(serde_json::from_slice(&bytes).with_context(|| {
                format!("Invalid state file {}", path.display())
            })?))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
pub fn write<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path.parent().context("Missing state directory")?;
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    serde_json::to_writer_pretty(temp.as_file_mut(), value)?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
pub fn token_path(server: &str) -> Result<PathBuf> {
    let directory = std::env::var_os("PROOFS_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::config_dir().map(|p| p.join("cargo-proofs")))
        .context("Cannot locate user config directory")?;
    Ok(directory.join(format!("{}.json", digest(server))))
}
#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub expires_at: u64,
}
#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub report_id: Option<u64>,
    pub revision: Option<u64>,
    pub snapshot: Option<Value>,
    pub pending: Option<Pending>,
    #[serde(default)]
    pub known_claims: std::collections::BTreeMap<String, Value>,
}
#[derive(Serialize, Deserialize)]
pub struct Pending {
    pub path: String,
    pub body: Value,
    pub key: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journal_round_trip() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("state.json");
        let s = State {
            pending: Some(Pending {
                path: "/api/v1/reports".into(),
                body: serde_json::json!({"title":"test"}),
                key: "retry-me".into(),
            }),
            ..State::default()
        };
        write(&p, &s).unwrap();
        let saved: State = read(&p).unwrap().unwrap();
        assert_eq!(saved.pending.unwrap().key, "retry-me");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&p).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
