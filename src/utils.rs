// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use crate::error::AuditCheckError;
use anyhow::Error;
use std::any::Any;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_join_error(_e: Box<dyn Any + Send>) -> Error {
    AuditCheckError::Join.into()
}
