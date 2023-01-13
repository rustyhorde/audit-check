// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use rustc_version::{Version, VersionMeta};

pub(crate) const MSRV: &str = "1.57.0";

pub(crate) fn check_rustc_version(meta: &VersionMeta) -> Result<bool> {
    Ok(meta.semver >= Version::parse(MSRV)?)
}

#[cfg(test)]
mod test {
    use super::check_rustc_version;
    use anyhow::Result;
    use rustc_version::{version_meta, version_meta_for};

    const OLD_RUSTC: &str = r#"rustc 1.56.1 (59eed8a2a 2021-11-01)
binary: rustc
commit-hash: 59eed8a2aac0230a8b53e89d4e99d55912ba6b35
commit-date: 2021-11-01
host: x86_64-unknown-linux-gnu
release: 1.56.1
LLVM version: 13.0.0
"#;

    #[test]
    fn check_rustc_version_succeeds() -> Result<()> {
        assert!(check_rustc_version(&version_meta()?).is_ok());
        Ok(())
    }

    #[test]
    fn check_rustc_version_fails() -> Result<()> {
        assert!(check_rustc_version(&version_meta_for(OLD_RUSTC)?).is_ok());
        Ok(())
    }
}
