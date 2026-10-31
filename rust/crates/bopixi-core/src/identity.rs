//! The built-in local identity used while `DISABLE_AUTH` is on: every request
//! acts as this admin, and `/api/auth/get-session` answers with a session for
//! it so the client opens straight on the dashboard. Mirrors
//! `shared/auth.ts` field for field.

use serde::Serialize;
use time::{format_description::FormatItem, macros::format_description, Duration, OffsetDateTime};

pub const LOCAL_USER_ID: &str = "local-user";
pub const LOCAL_SESSION_ID: &str = "local-session";
pub const LOCAL_SESSION_TOKEN: &str = "local-token";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUser {
    pub id: &'static str,
    pub name: &'static str,
    pub email: &'static str,
    pub role: &'static str,
    pub email_verified: bool,
    pub image: Option<&'static str>,
}

pub const DEFAULT_USER: LocalUser = LocalUser {
    id: LOCAL_USER_ID,
    name: "User",
    email: "user@local.dev",
    role: "admin",
    email_verified: true,
    image: None,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: &'static str,
    pub user_id: &'static str,
    pub token: &'static str,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct LocalSession {
    pub session: SessionInfo,
    pub user: LocalUser,
}

/// Milliseconds precision with a trailing `Z`, matching `Date#toISOString`.
const ISO_MILLIS: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

fn iso(t: OffsetDateTime) -> String {
    t.format(ISO_MILLIS)
        .expect("fixed ISO-8601 format cannot fail")
}

/// A fresh one-year session for the built-in local user, shaped exactly like
/// the JSON `localSession()` in `server/index.ts` hands the client.
pub fn local_session() -> LocalSession {
    let now = OffsetDateTime::now_utc();
    let expires = now + Duration::days(365);
    let created = iso(now);
    LocalSession {
        session: SessionInfo {
            id: LOCAL_SESSION_ID,
            user_id: LOCAL_USER_ID,
            token: LOCAL_SESSION_TOKEN,
            expires_at: iso(expires),
            created_at: created.clone(),
            updated_at: created,
        },
        user: DEFAULT_USER,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_the_better_auth_session_shape() {
        let value = serde_json::to_value(local_session()).unwrap();
        assert_eq!(value["user"]["id"], "local-user");
        assert_eq!(value["user"]["emailVerified"], true);
        assert_eq!(value["user"]["image"], serde_json::Value::Null);
        assert_eq!(value["session"]["userId"], "local-user");
        assert_eq!(value["session"]["token"], "local-token");
        let expires = value["session"]["expiresAt"].as_str().unwrap();
        assert!(expires.len() == 24 && expires.ends_with('Z'), "{expires}");
    }
}
