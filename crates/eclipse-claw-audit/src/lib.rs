//! Durable, privacy-preserving audit records for Eclipse Claw services.
//!
//! The store intentionally accepts a fixed operation identifier rather than a
//! raw URL, request body, user identifier, header, or secret. Files are JSONL,
//! append-only within a rotation segment, and private to the service account.
pub mod agent_plan;

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_RETENTION_DAYS: u64 = 14;
const MAX_RETENTION_DAYS: u64 = 90;
const DEFAULT_MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;
const MIN_MAX_FILE_BYTES: u64 = 64 * 1024;
const MAX_MAX_FILE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_RECENT_EVENTS: usize = 200;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("audit configuration error: {0}")]
    Configuration(String),
    #[error("audit I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("audit serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("audit store lock is poisoned")]
    Poisoned,
}

#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub directory: PathBuf,
    pub retention_days: u64,
    pub max_file_bytes: u64,
}

impl AuditConfig {
    pub fn from_env() -> Result<Option<Self>, AuditError> {
        let Some(directory) = std::env::var_os("ECLIPSE_AUDIT_DIR") else {
            return Ok(None);
        };
        if directory.is_empty() {
            return Err(AuditError::Configuration(
                "ECLIPSE_AUDIT_DIR must not be empty".into(),
            ));
        }

        let retention_days = parse_bounded_env(
            "ECLIPSE_AUDIT_RETENTION_DAYS",
            DEFAULT_RETENTION_DAYS,
            1,
            MAX_RETENTION_DAYS,
        )?;
        let max_file_bytes = parse_bounded_env(
            "ECLIPSE_AUDIT_MAX_FILE_BYTES",
            DEFAULT_MAX_FILE_BYTES,
            MIN_MAX_FILE_BYTES,
            MAX_MAX_FILE_BYTES,
        )?;

        Ok(Some(Self {
            directory: PathBuf::from(directory),
            retention_days,
            max_file_bytes,
        }))
    }
}

fn parse_bounded_env(
    name: &str,
    default: u64,
    minimum: u64,
    maximum: u64,
) -> Result<u64, AuditError> {
    let Some(raw) = std::env::var(name).ok() else {
        return Ok(default);
    };
    let value = raw.parse::<u64>().map_err(|_| {
        AuditError::Configuration(format!(
            "{name} must be an integer between {minimum} and {maximum}"
        ))
    })?;
    if !(minimum..=maximum).contains(&value) {
        return Err(AuditError::Configuration(format!(
            "{name} must be between {minimum} and {maximum}"
        )));
    }
    Ok(value)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub schema_version: u8,
    pub timestamp_unix_ms: u64,
    pub service: String,
    pub operation: String,
    pub outcome: String,
    pub status_code: u16,
    pub duration_ms: u64,
}

impl AuditEvent {
    pub fn new(
        service: impl Into<String>,
        operation: impl Into<String>,
        outcome: impl Into<String>,
        status_code: u16,
        duration_ms: u64,
    ) -> Self {
        Self {
            schema_version: 1,
            timestamp_unix_ms: unix_millis(),
            service: service.into(),
            operation: operation.into(),
            outcome: outcome.into(),
            status_code,
            duration_ms,
        }
    }
}

#[derive(Clone)]
pub struct AuditStore {
    inner: Arc<AuditStoreInner>,
}

struct AuditStoreInner {
    config: AuditConfig,
    write_lock: Mutex<()>,
}

impl AuditStore {
    pub fn open(config: AuditConfig) -> Result<Self, AuditError> {
        prepare_private_directory(&config.directory)?;
        let store = Self {
            inner: Arc::new(AuditStoreInner {
                config,
                write_lock: Mutex::new(()),
            }),
        };
        store.prune()?;
        Ok(store)
    }

    pub fn from_env(required: bool) -> Result<Option<Self>, AuditError> {
        match AuditConfig::from_env()? {
            Some(config) => Self::open(config).map(Some),
            None if required => Err(AuditError::Configuration(
                "ECLIPSE_AUDIT_REQUIRED=1 requires ECLIPSE_AUDIT_DIR".into(),
            )),
            None => Ok(None),
        }
    }

    pub fn retention_days(&self) -> u64 {
        self.inner.config.retention_days
    }

    pub fn record(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let _guard = self
            .inner
            .write_lock
            .lock()
            .map_err(|_| AuditError::Poisoned)?;
        let path = self.active_segment()?;
        reject_symlink(&path)?;
        let mut file = open_private_append(&path)?;
        serde_json::to_writer(&mut file, event)?;
        file.write_all(b"\n")?;
        file.flush()?;
        Ok(())
    }

    pub fn recent(&self, requested_limit: usize) -> Result<Vec<AuditEvent>, AuditError> {
        let limit = requested_limit.clamp(1, MAX_RECENT_EVENTS);
        let mut files = audit_files(&self.inner.config.directory)?;
        files.sort();
        files.reverse();

        let mut events = Vec::with_capacity(limit);
        for path in files {
            reject_symlink(&path)?;
            let reader = BufReader::new(File::open(path)?);
            let mut file_events = Vec::new();
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                file_events.push(serde_json::from_str::<AuditEvent>(&line)?);
            }
            for event in file_events.into_iter().rev() {
                events.push(event);
                if events.len() == limit {
                    events.reverse();
                    return Ok(events);
                }
            }
        }
        events.reverse();
        Ok(events)
    }

    pub fn prune(&self) -> Result<(), AuditError> {
        let today = unix_days();
        let oldest_allowed = today.saturating_sub(self.inner.config.retention_days - 1);
        for path in audit_files(&self.inner.config.directory)? {
            if segment_day(&path).is_some_and(|day| day < oldest_allowed) {
                reject_symlink(&path)?;
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn active_segment(&self) -> Result<PathBuf, AuditError> {
        let day = unix_days();
        for index in 0_u32..10_000 {
            let suffix = if index == 0 {
                String::new()
            } else {
                format!("-{index}")
            };
            let path = self
                .inner
                .config
                .directory
                .join(format!("audit-{day}{suffix}.jsonl"));
            match fs::metadata(&path) {
                Ok(metadata) if metadata.len() >= self.inner.config.max_file_bytes => continue,
                Ok(_) => return Ok(path),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(path),
                Err(error) => return Err(error.into()),
            }
        }
        Err(AuditError::Configuration(
            "audit rotation exhausted 10,000 segments for the current day".into(),
        ))
    }
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn unix_days() -> u64 {
    unix_millis() / 86_400_000
}

fn prepare_private_directory(path: &Path) -> Result<(), AuditError> {
    if path.exists() {
        reject_symlink(path)?;
        if !path.is_dir() {
            return Err(AuditError::Configuration(format!(
                "audit path is not a directory: {}",
                path.display()
            )));
        }
    } else {
        fs::create_dir_all(path)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), AuditError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(AuditError::Configuration(
            format!("audit path must not be a symlink: {}", path.display()),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn open_private_append(path: &Path) -> Result<File, AuditError> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}

fn audit_files(directory: &Path) -> Result<Vec<PathBuf>, AuditError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if segment_day(&path).is_some() {
            files.push(path);
        }
    }
    Ok(files)
}

fn segment_day(path: &Path) -> Option<u64> {
    let name = path.file_name()?.to_str()?;
    let body = name.strip_prefix("audit-")?.strip_suffix(".jsonl")?;
    body.split('-').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_directory(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "eclipse-claw-audit-{test_name}-{}-{}",
            std::process::id(),
            unix_millis()
        ))
    }

    fn store_at(path: PathBuf, max_file_bytes: u64) -> AuditStore {
        AuditStore::open(AuditConfig {
            directory: path,
            retention_days: 14,
            max_file_bytes,
        })
        .unwrap()
    }

    #[test]
    fn appends_and_reads_bounded_events_without_sensitive_fields() {
        let directory = temp_directory("append");
        let store = store_at(directory.clone(), DEFAULT_MAX_FILE_BYTES);
        store
            .record(&AuditEvent::new("server", "extract", "success", 200, 12))
            .unwrap();
        store
            .record(&AuditEvent::new("server", "summarise", "denied", 401, 1))
            .unwrap();

        let events = store.recent(1000).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].operation, "extract");
        let raw = serde_json::to_string(&events).unwrap();
        for forbidden in ["url", "query", "header", "cookie", "token", "body", "ip"] {
            assert!(!raw.contains(forbidden));
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rotates_when_the_active_segment_reaches_its_limit() {
        let directory = temp_directory("rotate");
        let store = store_at(directory.clone(), 1);
        store
            .record(&AuditEvent::new("worker", "complete", "success", 200, 2))
            .unwrap();
        store
            .record(&AuditEvent::new("worker", "complete", "success", 200, 3))
            .unwrap();
        assert_eq!(audit_files(&directory).unwrap().len(), 2);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn prunes_segments_outside_retention() {
        let directory = temp_directory("prune");
        fs::create_dir_all(&directory).unwrap();
        File::create(directory.join("audit-0.jsonl")).unwrap();
        let _store = store_at(directory.clone(), DEFAULT_MAX_FILE_BYTES);
        assert!(!directory.join("audit-0.jsonl").exists());
        fs::remove_dir_all(directory).unwrap();
    }
}
