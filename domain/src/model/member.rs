use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberEntity {
    pub account: String,
    pub password: String,
    pub email: Option<String>,
    pub jti: Option<String>,
    pub failed_attempts: i64,
    pub last_failed_at: Option<DateTime<Utc>>,
    pub last_signin_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl MemberEntity {
    pub fn signin_failed(&mut self) {
        self.failed_attempts += 1;
        self.jti = None;
        self.last_failed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn signup_success(&mut self, jti: &str) {
        self.jti = Some(jti.to_string());
        self.failed_attempts = 0;
        self.last_failed_at = None;
        self.last_signin_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn signout(&mut self) {
        self.jti = None;
        self.updated_at = Utc::now();
    }

    pub fn is_locked(&self, lock_threshold: i64, lock_seconds: i64) -> bool {
        if let Some(last_failed) = self.last_failed_at {
            if self.failed_attempts >= lock_threshold {
                let lock_duration = Duration::seconds(lock_seconds * self.failed_attempts);
                return Utc::now() < last_failed + lock_duration;
            }
        }
        false
    }

    pub fn is_busy(&self, interval_secs: i64) -> bool {
        Utc::now() >= self.updated_at + Duration::seconds(interval_secs)
    }
}
