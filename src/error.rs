// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

#[derive(Debug, Error)]
pub(crate) enum AuditCheckError {
    #[error("An error has occurred")]
    Anyhow(#[from] anyhow::Error),
    #[error("Unable to pipe stderr")]
    Stderr,
    #[error("Unable to pipe stdout")]
    Stdout,
    #[error("Unable to determine status code")]
    Code,
    #[error("Error joining thread handle")]
    Join,
    #[error("cargo audit version check failed")]
    AuditVersionCheck,
    #[error("cargo audit requires rust {msrv:?} or greater")]
    RustcVersionCheck { msrv: &'static str },
    #[error("failed to initialize tracing")]
    TryInit(#[from] TryInitError),
}
