use crate::notifications::outbox::OutboxEntry;
use jiff::Timestamp;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct NotificationOutboxEntryResponse {
    pub id: i64,
    pub provider: Option<String>,
    pub summary: Option<String>,
    pub team_id: Option<i64>,
    pub team: Option<String>,
    pub status: NotificationOutboxEntryStatus,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
    pub next_attempt_at: Timestamp,
}

impl From<OutboxEntry> for NotificationOutboxEntryResponse {
    fn from(entry: OutboxEntry) -> Self {
        let status = if entry.is_failing() {
            NotificationOutboxEntryStatus::Failing
        } else {
            NotificationOutboxEntryStatus::Queued
        };
        Self {
            id: entry.id,
            provider: entry.provider,
            summary: entry.summary,
            team_id: entry.team_id,
            team: entry.team,
            status,
            attempts: entry.attempts,
            last_error: entry.last_error,
            created_at: entry.created_at,
            next_attempt_at: entry.next_attempt_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotificationOutboxEntryStatus {
    Queued,
    Failing,
}
