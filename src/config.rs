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
    pub(crate) action_repo: String,
    pub(crate) create_issue: bool,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self> {
        // Error here as this is required, the others have defaults.
        let token = env::var("INPUT_TOKEN")?;
        let action_repo = env::var("GITHUB_ACTION_REPOSITORY")?;
        let deny = input_deny();
        let level = Level::from_str(&input_level())?;
        let create_issue = str::parse::<bool>(&input_create_issue())?;

        Ok(Self {
            token,
            deny,
            level,
            action_repo,
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
