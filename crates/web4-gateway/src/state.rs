use crate::config::RenderingConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{atomic::AtomicU64, Arc, Mutex};
use web4_core::{
    DefaultConsentEngine, ErrorCode, GatewayBindingExecutor, JsonSchemaValidator, RuntimeError,
    ServiceRuntime,
};

pub type RuntimeEngine =
    ServiceRuntime<JsonSchemaValidator, DefaultConsentEngine, GatewayBindingExecutor>;

#[derive(Debug, Clone)]
pub struct ChallengeRecord {
    pub service_id: String,
    pub expires_at: u64,
    pub status: ChallengeStatus,
    pub used: bool,
}

#[derive(Debug, Clone)]
pub struct RateWindow {
    pub window_started_at: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeStatus {
    Pending,
    Approved,
    Denied,
}

pub trait ChallengeStore: Send + Sync {
    fn insert(&self, challenge_id: String, record: ChallengeRecord) -> Result<(), RuntimeError>;
    fn update_status(
        &self,
        challenge_id: &str,
        status: ChallengeStatus,
        now: u64,
    ) -> Result<(), RuntimeError>;
    fn consume(&self, challenge_id: &str, service_id: &str, now: u64) -> Result<(), RuntimeError>;
}

pub trait RateLimiter: Send + Sync {
    fn enforce(
        &self,
        key: String,
        limit: u64,
        window_secs: u64,
        now: u64,
    ) -> Result<(), RuntimeError>;
}

#[derive(Default)]
pub struct InMemoryChallengeStore {
    records: Mutex<HashMap<String, ChallengeRecord>>,
}

impl ChallengeStore for InMemoryChallengeStore {
    fn insert(&self, challenge_id: String, record: ChallengeRecord) -> Result<(), RuntimeError> {
        let mut lock = self.records.lock().map_err(|_| {
            RuntimeError::new(
                ErrorCode::InternalError,
                "challenge store lock poisoned",
                false,
            )
        })?;
        lock.insert(challenge_id, record);
        Ok(())
    }

    fn update_status(
        &self,
        challenge_id: &str,
        status: ChallengeStatus,
        now: u64,
    ) -> Result<(), RuntimeError> {
        let mut lock = self.records.lock().map_err(|_| {
            RuntimeError::new(
                ErrorCode::InternalError,
                "challenge store lock poisoned",
                false,
            )
        })?;
        let record = lock.get_mut(challenge_id).ok_or_else(|| {
            RuntimeError::new(
                ErrorCode::NotFound,
                format!("challenge not found: {challenge_id}"),
                false,
            )
        })?;
        if record.expires_at <= now {
            return Err(RuntimeError::new(
                ErrorCode::InvalidArgument,
                "challenge expired",
                false,
            ));
        }
        record.status = status;
        Ok(())
    }

    fn consume(&self, challenge_id: &str, service_id: &str, now: u64) -> Result<(), RuntimeError> {
        let mut lock = self.records.lock().map_err(|_| {
            RuntimeError::new(
                ErrorCode::InternalError,
                "challenge store lock poisoned",
                false,
            )
        })?;
        let record = lock.get_mut(challenge_id).ok_or_else(|| {
            RuntimeError::new(
                ErrorCode::NotFound,
                format!("challenge not found: {challenge_id}"),
                false,
            )
        })?;

        if record.expires_at <= now {
            return Err(RuntimeError::new(
                ErrorCode::InvalidArgument,
                "challenge expired",
                false,
            ));
        }
        if record.service_id != service_id {
            return Err(RuntimeError::new(
                ErrorCode::Forbidden,
                "challenge-service mismatch",
                false,
            ));
        }
        if record.used {
            return Err(RuntimeError::new(
                ErrorCode::ConsentRequired,
                "challenge already consumed",
                false,
            ));
        }
        if record.status != ChallengeStatus::Approved {
            return Err(RuntimeError::new(
                ErrorCode::ConsentRequired,
                "challenge not approved",
                false,
            ));
        }

        record.used = true;
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemoryRateLimiter {
    windows: Mutex<HashMap<String, RateWindow>>,
}

impl RateLimiter for InMemoryRateLimiter {
    fn enforce(
        &self,
        key: String,
        limit: u64,
        window_secs: u64,
        now: u64,
    ) -> Result<(), RuntimeError> {
        let mut lock = self.windows.lock().map_err(|_| {
            RuntimeError::new(
                ErrorCode::InternalError,
                "rate limiter lock poisoned",
                false,
            )
        })?;
        let entry = lock.entry(key).or_insert(RateWindow {
            window_started_at: now,
            count: 0,
        });

        if now.saturating_sub(entry.window_started_at) >= window_secs {
            entry.window_started_at = now;
            entry.count = 0;
        }

        if entry.count >= limit {
            return Err(RuntimeError::new(
                ErrorCode::RateLimited,
                "rate limit exceeded",
                true,
            ));
        }

        entry.count += 1;
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppState {
    pub source: Arc<String>,
    pub model: Arc<Value>,
    pub document_entry: Arc<String>,
    pub runtime: Arc<RuntimeEngine>,
    pub jwt_secret: Arc<String>,
    pub challenge_counter: Arc<AtomicU64>,
    pub challenges: Arc<dyn ChallengeStore>,
    pub rate_limiter: Arc<dyn RateLimiter>,
    pub admin_token: Arc<String>,
    pub rendering: Arc<RenderingConfig>,
    pub document_root: Arc<PathBuf>,
    pub document_dir: Arc<PathBuf>,
}
