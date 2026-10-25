use crate::canary;
use reqwest::Client;
use sqlx::SqlitePool;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub(super) const CANARY_RECHECK_CACHE_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CanaryVerdict {
    Disabled,
    Reachable,
    Unreachable,
    Inconclusive,
}

impl CanaryVerdict {
    fn should_suppress_ambiguous_failure(self) -> bool {
        matches!(self, Self::Unreachable | Self::Inconclusive)
    }
}

pub(super) async fn run_and_record_canary(
    verifying_client: &Client,
    pool: &SqlitePool,
) -> CanaryVerdict {
    let settings = match canary::load_settings(pool).await {
        Ok(settings) => settings,
        Err(error) => {
            tracing::error!("Failed to load canary settings: {error}");
            return CanaryVerdict::Inconclusive;
        }
    };
    if !settings.config.enabled {
        return CanaryVerdict::Disabled;
    }
    let result = canary::check(verifying_client, &settings.config).await;
    if let Err(error) = &result {
        tracing::warn!("Canary check failed. Probe connectivity is degraded: {error}");
    }
    match canary::record_result_if_current(pool, settings.settings_revision, &result).await {
        Ok(canary::CanaryResultWrite::Recorded) => {
            if result.is_ok() {
                CanaryVerdict::Reachable
            } else {
                CanaryVerdict::Unreachable
            }
        }
        Ok(canary::CanaryResultWrite::DiscardedStaleRevision) => {
            tracing::warn!("Canary settings changed mid-check, so this probe run is abandoned");
            CanaryVerdict::Inconclusive
        }
        Err(error) => {
            tracing::error!("Failed to record canary result: {error}");
            if result.is_ok() {
                CanaryVerdict::Reachable
            } else {
                CanaryVerdict::Unreachable
            }
        }
    }
}

pub(super) struct CachedCanaryVerdict {
    pub(super) completed_at: Instant,
    pub(super) verdict: CanaryVerdict,
}

pub(super) struct AmbiguousFailureGuard<'a> {
    pub(super) initial_canary_verdict: CanaryVerdict,
    pub(super) cached_recheck: &'a Mutex<Option<CachedCanaryVerdict>>,
}

impl AmbiguousFailureGuard<'_> {
    pub(super) async fn should_suppress_ambiguous_failure(
        &self,
        verifying_client: &Client,
        pool: &SqlitePool,
    ) -> bool {
        let verdict = match self.initial_canary_verdict {
            CanaryVerdict::Disabled => return false,
            CanaryVerdict::Unreachable | CanaryVerdict::Inconclusive => return true,
            CanaryVerdict::Reachable => {
                self.recheck_coalescing_concurrent_callers(verifying_client, pool)
                    .await
            }
        };
        verdict.should_suppress_ambiguous_failure()
    }

    async fn recheck_coalescing_concurrent_callers(
        &self,
        verifying_client: &Client,
        pool: &SqlitePool,
    ) -> CanaryVerdict {
        let mut cached_recheck = self.cached_recheck.lock().await;
        if let Some(cached) = cached_recheck.as_ref()
            && cached.completed_at.elapsed() < CANARY_RECHECK_CACHE_TTL
        {
            return cached.verdict;
        }
        let verdict = run_and_record_canary(verifying_client, pool).await;
        *cached_recheck = Some(CachedCanaryVerdict {
            completed_at: Instant::now(),
            verdict,
        });
        verdict
    }
}
