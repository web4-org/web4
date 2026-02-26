use crate::error::GatewayError;
use crate::logic::common::unix_now;
use crate::state::{AppState, ChallengeRecord, ChallengeStatus};
use rand::RngCore;

pub fn create_challenge_record(
    state: &AppState,
    service_id: String,
    ttl_seconds: Option<u64>,
) -> Result<(String, u64), GatewayError> {
    let now = unix_now();
    let ttl = ttl_seconds.unwrap_or(120).min(600);
    let challenge_id = random_challenge_id(
        now,
        state
            .challenge_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    );
    let expires_at = now + ttl;
    state
        .challenges
        .insert(
            challenge_id.clone(),
            ChallengeRecord {
                service_id,
                expires_at,
                status: ChallengeStatus::Pending,
                used: false,
            },
        )
        .map_err(GatewayError)?;

    Ok((challenge_id, expires_at))
}

pub fn update_challenge_status(
    state: &AppState,
    challenge_id: &str,
    status: ChallengeStatus,
) -> Result<(), GatewayError> {
    state
        .challenges
        .update_status(challenge_id, status, unix_now())
        .map_err(GatewayError)
}

pub fn consume_interactive_challenge(
    state: &AppState,
    challenge_id: &str,
    service_id: &str,
) -> Result<(), GatewayError> {
    state
        .challenges
        .consume(challenge_id, service_id, unix_now())
        .map_err(GatewayError)
}

fn random_challenge_id(now: u64, sequence: u64) -> String {
    let mut entropy = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut entropy);
    let suffix: String = entropy.iter().map(|b| format!("{b:02x}")).collect();
    format!("ch-{now}-{sequence}-{suffix}")
}
