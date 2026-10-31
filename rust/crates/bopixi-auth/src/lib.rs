//! Request identity. `DISABLE_AUTH` is a development-only escape hatch that
//! resolves every request to the built-in local admin (`shared/auth.ts`).
//! Without it there is no actor yet: M3 adds better-auth-compatible session
//! verification, so until then a production server answers 401 everywhere
//! except the public routes.

use bopixi_core::{Env, DEFAULT_USER};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

impl Actor {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

pub fn local_actor() -> Actor {
    Actor {
        id: DEFAULT_USER.id.to_string(),
        name: DEFAULT_USER.name.to_string(),
        email: DEFAULT_USER.email.to_string(),
        role: DEFAULT_USER.role.to_string(),
    }
}

/// The actor for this request, or `None` (→ 401). The `Env` is consulted so
/// production cannot accidentally take the local-admin path: `Env::parse`
/// already refuses `DISABLE_AUTH=true` there.
pub fn resolve(env: &Env) -> Option<Actor> {
    if env.disable_auth() {
        Some(local_actor())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(pairs: &[(&str, &str)]) -> Env {
        Env::parse_from(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string()))).unwrap()
    }

    #[test]
    fn disable_auth_acts_as_the_local_admin() {
        let env = env_with(&[("DISABLE_AUTH", "true")]);
        let actor = resolve(&env).expect("local actor");
        assert_eq!(actor.id, "local-user");
        assert_eq!(actor.role, "admin");
        assert!(actor.is_admin());
    }

    #[test]
    fn without_disable_auth_there_is_no_actor_yet() {
        let env = env_with(&[("NODE_ENV", "test"), ("DISABLE_AUTH", "false")]);
        assert!(resolve(&env).is_none());
    }
}
