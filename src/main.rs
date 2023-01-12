// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::{anyhow, Result};
use check::{
    installed::check_audit,
    rustc::{check_rustc_version, MSRV},
};
use rustc_version::version_meta;
use std::env;

mod check;

fn main() -> Result<()> {
    if check_rustc_version(version_meta()?)? {
        if !check_audit("cargo audit --version")? {
            // TODO: Install 'cargo-audit'
            if let Ok(deny) = env::var("INPUTS_DENY") {
                println!("DENY: {deny}");
            }
        }
        Ok(())
    } else {
        Err(anyhow!("cargo audit requires rust {} or greater", MSRV))
    }
}
