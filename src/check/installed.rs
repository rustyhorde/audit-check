// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

pub(crate) fn check_audit<S>(command: S) -> Result<bool>
where
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("sh");
    let _ = cmd.arg("-c");
    let _ = cmd.arg(command);
    let _ = cmd.stdout(Stdio::piped());
    let _ = cmd.stderr(Stdio::piped());

    let out = cmd.output()?;
    Ok(out.status.success())
}

#[cfg(test)]
mod test {
    use super::check_audit;

    #[test]
    fn check_audit_fails() {
        let res = check_audit("blah -V");
        assert!(res.is_ok());
        assert!(!res.unwrap());
    }

    #[test]
    fn check_audit_succeeds() {
        let res = check_audit("rustc -Vv");
        assert!(res.is_ok());
        assert!(res.unwrap());
    }
}
