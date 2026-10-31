//! Environment parsing, ported from `server/env.ts`. Every variable the
//! server reads is declared once with its type and defaults; a bad value fails
//! the process at boot with a message naming the variable instead of surfacing
//! later as odd behaviour. Blank values (`X=` in a `.env`, an empty compose
//! entry) count as unset. `devDefault` applies whenever `NODE_ENV` is not
//! `production`.
//!
//! Two rules are stricter than the TypeScript original, both deliberate:
//! `DISABLE_AUTH` refuses to boot in production at all (the local admin
//! identity is a development escape hatch), and `DATABASE_URL` is required in
//! production because the embedded PGlite fallback does not exist in Rust.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeEnv {
    Development,
    Test,
    Production,
}

impl NodeEnv {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeEnv::Development => "development",
            NodeEnv::Test => "test",
            NodeEnv::Production => "production",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Str,
    Bool,
    Port,
    Origin,
    List,
    Int(i64),
    Choice(&'static [&'static str]),
}

struct VarDef {
    name: &'static str,
    kind: Kind,
    default: Option<&'static str>,
    dev_default: Option<&'static str>,
}

const STR: Kind = Kind::Str;
const BOOL: Kind = Kind::Bool;
const PORT: Kind = Kind::Port;
const ORIGIN: Kind = Kind::Origin;
const LIST: Kind = Kind::List;
const fn int(min: i64) -> Kind {
    Kind::Int(min)
}

macro_rules! v {
    ($name:literal, $kind:expr) => {
        VarDef {
            name: $name,
            kind: $kind,
            default: None,
            dev_default: None,
        }
    };
    ($name:literal, $kind:expr, default = $default:literal) => {
        VarDef {
            name: $name,
            kind: $kind,
            default: Some($default),
            dev_default: None,
        }
    };
    ($name:literal, $kind:expr, dev = $dev:literal) => {
        VarDef {
            name: $name,
            kind: $kind,
            default: None,
            dev_default: Some($dev),
        }
    };
    ($name:literal, $kind:expr, default = $default:literal, dev = $dev:literal) => {
        VarDef {
            name: $name,
            kind: $kind,
            default: Some($default),
            dev_default: Some($dev),
        }
    };
}

#[rustfmt::skip]
static SCHEMA: &[VarDef] = &[
    v!("NODE_ENV", Kind::Choice(&["development", "test", "production"]), default = "development"),
    v!("PORT", PORT, default = "4400"),
    v!("DATABASE_URL", STR),

    /* auth */
    v!("DISABLE_AUTH", BOOL, default = "false", dev = "true"),
    v!("BETTER_AUTH_SECRET", STR, dev = "bopixi-dev-secret-not-for-production"),
    v!("BETTER_AUTH_URL", ORIGIN, dev = "http://localhost:4300"),
    v!("TRUSTED_ORIGINS", LIST, default = ""),
    v!("SIGNUP_EMAIL_DOMAINS", LIST, default = ""),
    v!("ADMIN_EMAILS", LIST, default = ""),
    v!("REQUIRE_EMAIL_VERIFICATION", BOOL, default = "true"),
    v!("OIDC_ISSUER", STR),
    v!("OIDC_CLIENT_ID", STR),
    v!("OIDC_CLIENT_SECRET", STR),
    v!("OIDC_SCOPES", STR),
    v!("OIDC_PROVIDER_NAME", STR),
    v!("GOOGLE_CLIENT_ID", STR),
    v!("GOOGLE_CLIENT_SECRET", STR),
    v!("MICROSOFT_CLIENT_ID", STR),
    v!("MICROSOFT_CLIENT_SECRET", STR),
    v!("MICROSOFT_TENANT_ID", STR, default = "common"),
    v!("APPLE_CLIENT_ID", STR),
    v!("APPLE_CLIENT_SECRET", STR),
    v!("APPLE_APP_BUNDLE_IDENTIFIER", STR),
    v!("APPLE_JWKS_URL", STR, default = "https://appleid.apple.com/auth/keys"),

    /* email */
    v!("SMTP_HOST", STR),
    v!("SMTP_PORT", PORT, default = "587"),
    v!("SMTP_USER", STR),
    v!("SMTP_PASS", STR),
    v!("SMTP_SECURE", BOOL, default = "false"),
    v!("EMAIL_FROM", STR, default = "bopixi <no-reply@localhost>"),

    /* webhooks */
    v!("WEBHOOK_ALLOW_PRIVATE_URLS", BOOL, default = "false"),

    /* resident agents */
    v!("BOPIXI_AGENT_PROVIDER", Kind::Choice(&["anthropic", "azure"]), default = "anthropic"),
    v!("ANTHROPIC_API_KEY", STR),
    v!("AZURE_OPENAI_ENDPOINT", STR),
    v!("AZURE_OPENAI_API_KEY", STR),
    v!("AZURE_OPENAI_DEPLOYMENT", STR),
    v!("AZURE_OPENAI_API_VERSION", STR),
    v!("AZURE_OPENAI_REASONING_EFFORT", STR),
    v!("RESIDENT_TASK_LIMIT", int(0), default = "0"),
    v!("BOPIXI_AGENT_MODEL", STR, default = "claude-opus-5"),
    v!("BOPIXI_DISTILL_MODEL", STR, default = "claude-haiku-4-5-20251001"),
    v!("BOPIXI_AGENT_OPENAI_MODEL", STR, default = "gpt-5.6-terra"),
    v!("BOPIXI_AGENT_OPENAI_EFFORT", STR, default = "medium"),
    v!("BOPIXI_AGENT_OPENROUTER_MODEL", STR, default = "moonshotai/kimi-k2.6"),
    v!("BOPIXI_AGENT_GEMINI_MODEL", STR, default = "gemini-3.7-flash"),
    v!("BOPIXI_GEMINI_CLOUD_WORKERS", STR),
    v!("CHATGPT_CONNECT_DISABLED", BOOL, default = "false"),
    v!("CHATGPT_OAUTH_ISSUER", ORIGIN, default = "https://auth.openai.com"),
    v!("CHATGPT_OAUTH_CLIENT_ID", STR, default = "app_EMoamEEZ73f0CkXaXp7hrann"),
    v!("CHATGPT_OAUTH_REDIRECT_URI", STR, default = "http://localhost:1455/auth/callback"),
    v!("CHATGPT_OAUTH_CALLBACK_PORT", PORT, default = "1455"),
    v!("CHATGPT_AUTH_USER_AGENT", STR, default = "bopixi/0.1 (+https://doop.design)"),
    v!("CHATGPT_DEVICE_URL", STR),
    v!("CHATGPT_RESPONSES_URL", STR, default = "https://chatgpt.com/backend-api/codex/responses"),
    v!("CHATGPT_MODELS_URL", STR, default = "https://chatgpt.com/backend-api/codex/models"),
    v!("CHATGPT_CODEX_CLIENT_VERSION", STR, default = "0.163.0"),
    v!("CHATGPT_ORIGINATOR", STR, default = "codex_cli_rs"),
    v!("OPENAI_RESPONSES_URL", STR, default = "https://api.openai.com/v1/responses"),
    v!("OPENAI_MODELS_URL", STR, default = "https://api.openai.com/v1/models"),
    v!("OPENROUTER_CHAT_URL", STR, default = "https://openrouter.ai/api/v1/chat/completions"),
    v!("GEMINI_OPENAI_CHAT_URL", STR, default = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"),

    /* image generation */
    v!("OPENAI_API_KEY", STR),
    v!("GEMINI_API_KEY", STR),
    v!("ARK_API_KEY", STR),
    v!("ARK_IMAGES_URL", STR, default = "https://ark.ap-southeast.bytepluses.com/api/v3/images/generations"),
    v!("GEMINI_MODELS_URL", STR, default = "https://generativelanguage.googleapis.com/v1beta/models"),
    v!("BOPIXI_IMAGE_MODEL", STR, default = "gpt-image-2"),
    v!("BOPIXI_IMAGE_ROUTER_MODEL", STR),

    /* integrations */
    v!("META_APP_ID", STR),
    v!("META_APP_SECRET", STR),
    v!("LINEAR_CLIENT_ID", STR),
    v!("LINEAR_CLIENT_SECRET", STR),
    v!("LINEAR_WEBHOOK_SECRET", STR),
    v!("LINEAR_API_URL", STR, default = "https://api.linear.app/graphql"),
    v!("LINEAR_TOKEN_URL", STR, default = "https://api.linear.app/oauth/token"),
    v!("SLACK_CLIENT_ID", STR),
    v!("SLACK_CLIENT_SECRET", STR),
    v!("SLACK_SIGNING_SECRET", STR),
    v!("SLACK_API_URL", STR, default = "https://slack.com/api"),
    v!("GITHUB_APP_ID", STR),
    v!("GITHUB_APP_SLUG", STR),
    v!("GITHUB_APP_CLIENT_ID", STR),
    v!("GITHUB_APP_CLIENT_SECRET", STR),
    v!("GITHUB_APP_PRIVATE_KEY", STR),
    v!("PEXELS_API_KEY", STR),
    v!("BRANDFETCH_CLIENT_ID", STR),
    v!("CONTEXT_DEV_API_KEY", STR),

    /* object storage */
    v!("BUCKET", STR),
    v!("ACCESS_KEY_ID", STR),
    v!("SECRET_ACCESS_KEY", STR),
    v!("ENDPOINT", STR),
    v!("REGION", STR, default = "auto"),

    /* billing */
    v!("STRIPE_SECRET_KEY", STR),
    v!("STRIPE_WEBHOOK_SECRET", STR),
    v!("STRIPE_PRICE_TEAM_MONTHLY", STR),
    v!("STRIPE_PRICE_TEAM_YEARLY", STR),

    /* screenshots */
    v!("CHROME_PATH", STR),
    v!("CHROME_NO_SANDBOX", BOOL, default = "false"),

    /* analytics */
    v!("POSTHOG_KEY", STR),
    v!("VITE_POSTHOG_KEY", STR),
    v!("POSTHOG_INGEST_HOST", ORIGIN, default = "https://us.i.posthog.com"),
    v!("POSTHOG_ASSETS_HOST", ORIGIN, default = "https://us-assets.i.posthog.com"),

    /* backgrounds */
    v!("BACKGROUNDS_ORIGIN", ORIGIN),

    /* rate limits */
    v!("MCP_UPLOADS_PER_MIN", int(1), default = "15"),
    v!("MCP_SEARCHES_PER_MIN", int(1), default = "12"),
    v!("MCP_GENERATIONS_PER_MIN", int(1), default = "6"),
    v!("MCP_IMPORTS_PER_MIN", int(1), default = "5"),
    v!("SYNC_INGESTS_PER_MIN", int(1), default = "30"),
    v!("PREVIEW_RENDERS_PER_MIN", int(1), default = "12"),
    v!("SYNC_PAGE_INTERVAL_MS", int(0), default = "30000"),
    v!("SYNC_FREEZE_MS", int(0), default = "900000"),

    /* iOS push */
    v!("BOPIXI_APNS_TEAM_ID", STR),
    v!("BOPIXI_APNS_KEY_ID", STR),
    v!("BOPIXI_APNS_PRIVATE_KEY", STR),
    v!("BOPIXI_APNS_BUNDLE_ID", STR, default = "design.bopixi.ios"),
];

fn def(name: &str) -> Option<&'static VarDef> {
    SCHEMA.iter().find(|d| d.name == name)
}

#[derive(Debug)]
pub struct EnvError {
    pub lines: Vec<String>,
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid environment configuration:")?;
        for line in &self.lines {
            write!(f, "\n  {line}")?;
        }
        Ok(())
    }
}

impl std::error::Error for EnvError {}

#[derive(Debug)]
pub struct Env {
    node_env: NodeEnv,
    values: BTreeMap<&'static str, String>,
    lists: BTreeMap<&'static str, Vec<String>>,
}

enum Validated {
    Value(String),
    List(Vec<String>),
}

fn invalid(message: impl Into<String>) -> String {
    message.into()
}

fn validate(kind: Kind, raw: &str) -> Result<Validated, String> {
    match kind {
        Kind::Str => Ok(Validated::Value(raw.to_string())),
        Kind::Bool => match raw {
            "true" | "1" => Ok(Validated::Value("true".to_string())),
            "false" | "0" => Ok(Validated::Value("false".to_string())),
            _ => Err(invalid(format!("Invalid bool: \"{raw}\""))),
        },
        Kind::Port => match raw.trim().parse::<i64>() {
            Ok(n) if (0..=65535).contains(&n) => Ok(Validated::Value(n.to_string())),
            _ => Err(invalid(format!("Invalid port: \"{raw}\""))),
        },
        Kind::Origin => {
            let trimmed = raw.trim();
            match url::Url::parse(trimmed) {
                Ok(u) if u.scheme() == "http" || u.scheme() == "https" => {
                    Ok(Validated::Value(trimmed.trim_end_matches('/').to_string()))
                }
                _ => Err(invalid(format!(
                    "Invalid URL (expected http or https): \"{raw}\""
                ))),
            }
        }
        Kind::List => Ok(Validated::List(
            raw.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
        )),
        Kind::Int(min) => match raw.trim().parse::<i64>() {
            Ok(n) if n >= min => Ok(Validated::Value(n.to_string())),
            _ => Err(invalid(format!(
                "Invalid integer (expected >= {min}): \"{raw}\""
            ))),
        },
        Kind::Choice(choices) => {
            if choices.contains(&raw) {
                Ok(Validated::Value(raw.to_string()))
            } else {
                Err(invalid(format!(
                    "Invalid choice: \"{raw}\" (expected one of: {})",
                    choices.join(", ")
                )))
            }
        }
    }
}

impl Env {
    /// Parse the process environment. Use [`Env::parse_from`] in tests.
    pub fn parse() -> Result<Self, EnvError> {
        Self::parse_from(std::env::vars_os().map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.to_string_lossy().into_owned(),
            )
        }))
    }

    pub fn parse_from<I, K, V>(pairs: I) -> Result<Self, EnvError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut raw: BTreeMap<String, String> = pairs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .filter(|(_, v)| !v.trim().is_empty())
            .collect();

        /* these keys were DOOP_* before the rename; an explicit BOPIXI_*
        value wins, so an un-updated environment keeps booting */
        for d in SCHEMA {
            let Some(suffix) = d.name.strip_prefix("BOPIXI_") else {
                continue;
            };
            if raw.contains_key(d.name) {
                continue;
            }
            if let Some(legacy) = raw.get(&format!("DOOP_{suffix}")).cloned() {
                raw.insert(d.name.to_string(), legacy);
            }
        }

        /* envalid applies devDefault whenever NODE_ENV is not production; an
        unset (or unparseable) NODE_ENV therefore behaves like development */
        let production = raw.get("NODE_ENV").map(String::as_str) == Some("production");

        let mut errors: Vec<String> = Vec::new();
        let mut values: BTreeMap<&'static str, String> = BTreeMap::new();
        let mut lists: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

        for d in SCHEMA {
            /* envalid: outside production devDefault replaces default */
            let effective: Option<&str> = match raw.get(d.name) {
                Some(value) => Some(value.as_str()),
                None => {
                    if production {
                        d.default
                    } else {
                        d.dev_default.or(d.default)
                    }
                }
            };
            let Some(effective) = effective else {
                if production && d.default.is_none() && d.dev_default.is_some() {
                    errors.push(format!("{}: missing", d.name));
                }
                continue;
            };
            match validate(d.kind, effective) {
                Ok(Validated::Value(value)) => {
                    values.insert(d.name, value);
                }
                Ok(Validated::List(items)) => {
                    lists.insert(d.name, items);
                    values.insert(d.name, effective.to_string());
                }
                Err(message) => errors.push(format!("{}: {message}", d.name)),
            }
        }

        let node_env = match values.get("NODE_ENV").map(String::as_str) {
            Some("test") => NodeEnv::Test,
            Some("production") => NodeEnv::Production,
            _ => NodeEnv::Development,
        };

        if production && !values.contains_key("DATABASE_URL") {
            errors.push(
                "DATABASE_URL: missing (the Rust server needs an external Postgres; the embedded PGlite fallback is not supported)"
                    .to_string(),
            );
        }
        if production && values.get("DISABLE_AUTH").map(String::as_str) == Some("true") {
            errors.push(
                "DISABLE_AUTH: refusing to start in production with authentication disabled"
                    .to_string(),
            );
        }

        if errors.is_empty() {
            Ok(Env {
                node_env,
                values,
                lists,
            })
        } else {
            Err(EnvError { lines: errors })
        }
    }

    pub fn node_env(&self) -> NodeEnv {
        self.node_env
    }

    pub fn is_dev(&self) -> bool {
        self.node_env != NodeEnv::Production
    }

    pub fn is_production(&self) -> bool {
        self.node_env == NodeEnv::Production
    }

    pub fn get_str(&self, name: &str) -> Option<&str> {
        self.known(name);
        self.values.get(name).map(String::as_str)
    }

    pub fn get_bool(&self, name: &str) -> bool {
        self.known(name);
        match self.values.get(name).map(String::as_str) {
            Some("true") => true,
            Some("false") => false,
            other => panic!("{name} is not a parsed boolean: {other:?}"),
        }
    }

    pub fn get_int(&self, name: &str) -> i64 {
        self.known(name);
        self.values
            .get(name)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("{name} is not a parsed integer"))
    }

    pub fn get_list(&self, name: &str) -> &[String] {
        self.known(name);
        self.lists.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    fn known(&self, name: &str) {
        assert!(def(name).is_some(), "unknown environment variable {name}");
    }

    pub fn port(&self) -> u16 {
        self.get_int("PORT") as u16
    }

    pub fn disable_auth(&self) -> bool {
        self.get_bool("DISABLE_AUTH")
    }

    pub fn database_url(&self) -> Option<&str> {
        self.get_str("DATABASE_URL")
    }

    pub fn better_auth_secret(&self) -> &str {
        self.get_str("BETTER_AUTH_SECRET")
            .expect("BETTER_AUTH_SECRET is guaranteed by parse in every environment")
    }

    pub fn better_auth_url(&self) -> &str {
        self.get_str("BETTER_AUTH_URL")
            .expect("BETTER_AUTH_URL is guaranteed by parse in every environment")
    }

    pub fn trusted_origins(&self) -> &[String] {
        self.get_list("TRUSTED_ORIGINS")
    }

    pub fn admin_emails(&self) -> &[String] {
        self.get_list("ADMIN_EMAILS")
    }

    /// Stripe configured on this server — the same test `billingEnabled()`
    /// makes in `server/billing.ts`.
    pub fn billing_enabled(&self) -> bool {
        self.get_str("STRIPE_SECRET_KEY").is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn parse(pairs: &[(&str, &str)]) -> Result<Env, EnvError> {
        Env::parse_from(pairs.iter().copied())
    }

    fn dev() -> Env {
        parse(&[("NODE_ENV", "development")]).expect("development env parses")
    }

    #[test]
    fn parses_numbers_booleans_lists_and_origins() {
        let env = parse(&[
            ("NODE_ENV", "development"),
            ("MCP_UPLOADS_PER_MIN", "2"),
            ("WEBHOOK_ALLOW_PRIVATE_URLS", "1"),
            ("TRUSTED_ORIGINS", " https://a.example, https://b.example ,"),
            ("BETTER_AUTH_URL", "https://bopixi.example/"),
        ])
        .unwrap();
        assert_eq!(env.get_int("MCP_UPLOADS_PER_MIN"), 2);
        assert!(env.get_bool("WEBHOOK_ALLOW_PRIVATE_URLS"));
        assert_eq!(
            env.trusted_origins(),
            &[
                "https://a.example".to_string(),
                "https://b.example".to_string()
            ]
        );
        assert_eq!(env.better_auth_url(), "https://bopixi.example");
    }

    #[test]
    fn legacy_doop_prefixed_keys_still_apply() {
        let env = parse(&[
            ("NODE_ENV", "development"),
            ("DOOP_AGENT_PROVIDER", "azure"),
            ("DOOP_AGENT_MODEL", "legacy-model"),
        ])
        .unwrap();
        assert_eq!(env.get_str("BOPIXI_AGENT_PROVIDER"), Some("azure"));
        assert_eq!(env.get_str("BOPIXI_AGENT_MODEL"), Some("legacy-model"));

        /* an explicit new name wins over the old one */
        let env = parse(&[
            ("NODE_ENV", "development"),
            ("DOOP_AGENT_PROVIDER", "azure"),
            ("BOPIXI_AGENT_PROVIDER", "anthropic"),
        ])
        .unwrap();
        assert_eq!(env.get_str("BOPIXI_AGENT_PROVIDER"), Some("anthropic"));
    }

    #[test]
    fn blank_values_mean_unset() {
        let env = parse(&[
            ("NODE_ENV", "development"),
            ("MCP_UPLOADS_PER_MIN", ""),
            ("SMTP_SECURE", "   "),
            ("OPENAI_API_KEY", ""),
        ])
        .unwrap();
        assert_eq!(env.get_int("MCP_UPLOADS_PER_MIN"), 15);
        assert!(!env.get_bool("SMTP_SECURE"));
        assert_eq!(env.get_str("OPENAI_API_KEY"), None);
    }

    #[test]
    fn refuses_a_bad_value_and_names_the_variable() {
        let err = parse(&[("MCP_UPLOADS_PER_MIN", "1O")]).unwrap_err();
        assert!(
            err.to_string()
                .contains("MCP_UPLOADS_PER_MIN: Invalid integer"),
            "{err}"
        );

        let err = parse(&[("MCP_UPLOADS_PER_MIN", "0")]).unwrap_err();
        assert!(err.to_string().contains("MCP_UPLOADS_PER_MIN"), "{err}");

        let err = parse(&[("SMTP_SECURE", "maybe")]).unwrap_err();
        assert!(
            err.to_string().contains("SMTP_SECURE: Invalid bool"),
            "{err}"
        );

        let err = parse(&[("BETTER_AUTH_URL", "bopixi.example")]).unwrap_err();
        assert!(
            err.to_string().contains("BETTER_AUTH_URL: Invalid URL"),
            "{err}"
        );

        let err = parse(&[("BOPIXI_AGENT_PROVIDER", "openai")]).unwrap_err();
        assert!(err.to_string().contains("BOPIXI_AGENT_PROVIDER"), "{err}");
    }

    #[test]
    fn reports_every_bad_variable_in_one_error() {
        let err = parse(&[("PORT", "eighty"), ("CHROME_NO_SANDBOX", "yes please")]).unwrap_err();
        let text = err.to_string();
        assert!(text.contains("PORT: "), "{text}");
        assert!(text.contains("CHROME_NO_SANDBOX: "), "{text}");
        assert_eq!(text.lines().count(), 3, "header plus two problems: {text}");
    }

    #[test]
    fn defaults_an_unset_node_env_to_development() {
        let env = Env::parse_from(std::iter::empty::<(&str, &str)>()).unwrap();
        assert_eq!(env.node_env(), NodeEnv::Development);
        assert!(env.is_dev());
        assert!(!env.is_production());
    }

    #[test]
    fn requires_the_auth_secret_and_origin_in_production_only() {
        let env = dev();
        assert_eq!(
            env.better_auth_secret(),
            "bopixi-dev-secret-not-for-production"
        );
        assert_eq!(env.better_auth_url(), "http://localhost:4300");

        let err = Env::parse_from([
            ("NODE_ENV", "production"),
            ("DATABASE_URL", "postgres://bopixi:bopixi@127.0.0.1:5432/bopixi"),
        ])
        .unwrap_err();
        let text = err.to_string();
        assert!(text.contains("BETTER_AUTH_SECRET: missing"), "{text}");
        assert!(text.contains("BETTER_AUTH_URL: missing"), "{text}");

        let env = Env::parse_from([
            ("NODE_ENV", "production"),
            ("DATABASE_URL", "postgres://bopixi:bopixi@127.0.0.1:5432/bopixi"),
            ("BETTER_AUTH_SECRET", "a-real-secret"),
            ("BETTER_AUTH_URL", "https://bopixi.example"),
        ])
        .unwrap();
        assert!(env.is_production());
    }

    #[test]
    fn disable_auth_defaults_on_in_development_and_off_in_production() {
        assert!(dev().disable_auth());

        let env = Env::parse_from([
            ("NODE_ENV", "production"),
            ("DATABASE_URL", "postgres://bopixi:bopixi@127.0.0.1:5432/bopixi"),
            ("BETTER_AUTH_SECRET", "a-real-secret"),
            ("BETTER_AUTH_URL", "https://bopixi.example"),
        ])
        .unwrap();
        assert!(!env.disable_auth());
    }

    #[test]
    fn refuses_disable_auth_in_production() {
        let err = Env::parse_from([
            ("NODE_ENV", "production"),
            ("DISABLE_AUTH", "true"),
            ("DATABASE_URL", "postgres://bopixi:bopixi@127.0.0.1:5432/bopixi"),
            ("BETTER_AUTH_SECRET", "a-real-secret"),
            ("BETTER_AUTH_URL", "https://bopixi.example"),
        ])
        .unwrap_err();
        assert!(err.to_string().contains("DISABLE_AUTH"), "{err}");
    }

    #[test]
    fn requires_database_url_in_production() {
        let err = Env::parse_from([
            ("NODE_ENV", "production"),
            ("BETTER_AUTH_SECRET", "a-real-secret"),
            ("BETTER_AUTH_URL", "https://bopixi.example"),
        ])
        .unwrap_err();
        assert!(err.to_string().contains("DATABASE_URL: missing"), "{err}");
    }

    #[test]
    #[should_panic(expected = "unknown environment variable MCP_UPLOAD_PER_MIN")]
    fn a_typo_cannot_read_as_unset() {
        dev().get_str("MCP_UPLOAD_PER_MIN");
    }

    #[test]
    fn documents_every_declared_variable_in_env_example() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.env.example");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let documented: std::collections::HashSet<&str> = text
            .lines()
            .filter_map(|line| {
                let line = line.strip_prefix('#').unwrap_or(line);
                let (key, _) = line.split_once('=')?;
                if key.is_empty()
                    || !key
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                    || !key.starts_with(|c: char| c.is_ascii_uppercase())
                {
                    return None;
                }
                Some(key)
            })
            .collect();
        let missing: Vec<&str> = SCHEMA
            .iter()
            .map(|d| d.name)
            .filter(|name| !documented.contains(name))
            .collect();
        assert!(
            missing.is_empty(),
            "undocumented in .env.example: {missing:?}"
        );
    }
}
