// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use time::format_description::well_known::Iso8601;
use tracing::{metadata::LevelFilter, Level};
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    prelude::__tracing_subscriber_SubscriberExt,
    registry,
    util::SubscriberInitExt,
};

pub(crate) fn initialize(level: Level) -> Result<()> {
    let format = fmt::layer()
        .compact()
        .with_level(true)
        .with_ansi(true)
        .with_target(false)
        .with_timer(UtcTime::new(Iso8601::DEFAULT));
    let filter_layer = LevelFilter::from(level);
    Ok(registry().with(format).with(filter_layer).try_init()?)
}
