// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use std::{env, str::FromStr};
use tracing::Level;

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) token: String,
    pub(crate) deny: String,
    pub(crate) level: Level,
    pub(crate) owner_repo: String,
    pub(crate) create_issue: bool,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self> {
        // Error here as this is required, the others have defaults.
        let token = env::var("INPUT_TOKEN")?;
        let owner_repo = env::var("GITHUB_REPOSITORY")?;
        let deny = input_deny();
        let level = Level::from_str(&input_level())?;
        let create_issue = str::parse::<bool>(&input_create_issue())?;

        Ok(Self {
            token,
            deny,
            level,
            owner_repo,
            create_issue,
        })
    }
}

fn input_level() -> String {
    env::var("INPUT_LEVEL").unwrap_or_else(|_| "INFO".to_string())
}

fn input_deny() -> String {
    env::var("INPUT_DENY").unwrap_or_else(|_| "warnings".to_string())
}

fn input_create_issue() -> String {
    env::var("INPUT_CREATE_ISSUE").unwrap_or_else(|_| "false".to_string())
}

#[cfg(test)]
#[allow(unsafe_code)]
mod test {
    use super::Config;
    use std::sync::{LazyLock, Mutex};

    static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn config_defaults() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("INPUT_TOKEN", "tok");
            std::env::set_var("GITHUB_REPOSITORY", "owner/repo");
            std::env::remove_var("INPUT_LEVEL");
            std::env::remove_var("INPUT_DENY");
            std::env::remove_var("INPUT_CREATE_ISSUE");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.token, "tok");
        assert_eq!(config.owner_repo, "owner/repo");
        assert_eq!(config.deny, "warnings");
        assert_eq!(config.level, tracing::Level::INFO);
        assert!(!config.create_issue);
        unsafe {
            std::env::remove_var("INPUT_TOKEN");
            std::env::remove_var("GITHUB_REPOSITORY");
        }
    }

    #[test]
    fn config_all_vars() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("INPUT_TOKEN", "mytoken");
            std::env::set_var("GITHUB_REPOSITORY", "org/project");
            std::env::set_var("INPUT_LEVEL", "DEBUG");
            std::env::set_var("INPUT_DENY", "unsound");
            std::env::set_var("INPUT_CREATE_ISSUE", "true");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.token, "mytoken");
        assert_eq!(config.owner_repo, "org/project");
        assert_eq!(config.deny, "unsound");
        assert_eq!(config.level, tracing::Level::DEBUG);
        assert!(config.create_issue);
        unsafe {
            std::env::remove_var("INPUT_TOKEN");
            std::env::remove_var("GITHUB_REPOSITORY");
            std::env::remove_var("INPUT_LEVEL");
            std::env::remove_var("INPUT_DENY");
            std::env::remove_var("INPUT_CREATE_ISSUE");
        }
    }

    #[test]
    fn config_missing_token_fails() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::remove_var("INPUT_TOKEN");
            std::env::set_var("GITHUB_REPOSITORY", "owner/repo");
        }
        assert!(Config::from_env().is_err());
        unsafe {
            std::env::remove_var("GITHUB_REPOSITORY");
        }
    }

    #[test]
    fn config_missing_repo_fails() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("INPUT_TOKEN", "tok");
            std::env::remove_var("GITHUB_REPOSITORY");
        }
        assert!(Config::from_env().is_err());
        unsafe {
            std::env::remove_var("INPUT_TOKEN");
        }
    }

    #[test]
    fn config_invalid_level_fails() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("INPUT_TOKEN", "tok");
            std::env::set_var("GITHUB_REPOSITORY", "owner/repo");
            std::env::set_var("INPUT_LEVEL", "INVALID");
        }
        assert!(Config::from_env().is_err());
        unsafe {
            std::env::remove_var("INPUT_TOKEN");
            std::env::remove_var("GITHUB_REPOSITORY");
            std::env::remove_var("INPUT_LEVEL");
        }
    }

    #[test]
    fn config_invalid_create_issue_fails() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("INPUT_TOKEN", "tok");
            std::env::set_var("GITHUB_REPOSITORY", "owner/repo");
            std::env::set_var("INPUT_CREATE_ISSUE", "yes");
        }
        assert!(Config::from_env().is_err());
        unsafe {
            std::env::remove_var("INPUT_TOKEN");
            std::env::remove_var("GITHUB_REPOSITORY");
            std::env::remove_var("INPUT_CREATE_ISSUE");
        }
    }
}
