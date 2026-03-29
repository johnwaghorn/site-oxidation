use anyhow::{Context, bail};
use std::fs;
use std::io::Write;
use std::path::Path;
use tower_sessions::cookie::Key;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const SESSION_KEY_LEN: usize = 64;

#[cfg(unix)]
const PERMISSION_MASK: u32 = 0o777;
#[cfg(unix)]
const OWNER_READ_ONLY: u32 = 0o400;
#[cfg(unix)]
const OWNER_READ_WRITE: u32 = 0o600;

pub fn get_or_create_key(key_path: &Path, data_dir: &Path) -> anyhow::Result<Key> {
    if key_path.exists() {
        #[cfg(unix)]
        {
            let mode = fs::metadata(key_path)
                .context("Failed to stat session key file")?
                .permissions()
                .mode()
                & PERMISSION_MASK;
            if !matches!(mode, OWNER_READ_ONLY | OWNER_READ_WRITE) {
                bail!("Session key file must have 0400 or 0600 permissions, got {mode:04o}");
            }
        }
        let key_bytes = fs::read(key_path).context("Failed to read session key")?;
        if key_bytes.len() != SESSION_KEY_LEN {
            bail!(
                "Session key file has invalid length {} (expected {SESSION_KEY_LEN})",
                key_bytes.len()
            );
        }
        Ok(Key::from(&key_bytes))
    } else {
        let key = Key::generate();
        fs::create_dir_all(data_dir).context("Failed to create data directory")?;
        let mut open_options = fs::OpenOptions::new();
        open_options.write(true).create_new(true);
        #[cfg(unix)]
        open_options.mode(OWNER_READ_WRITE);
        let mut key_file = open_options
            .open(key_path)
            .context("Failed to create session key file")?;
        key_file
            .write_all(key.master())
            .context("Failed to write session key")?;
        Ok(key)
    }
}
