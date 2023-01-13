// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use crate::{
    audit::audit,
    check::{
        installed::check_audit,
        rustc::{check_rustc_version, MSRV},
    },
    config::Config,
    error::AuditCheckError,
    log::initialize,
    utils::handle_join_error,
};
use anyhow::Result;
use rustc_version::version_meta;
use std::{
    sync::mpsc::{channel, Receiver},
    thread::spawn,
};
use tracing::{error, info};

pub(crate) fn run() -> Result<()> {
    let config = Config::from_env()?;
    initialize(config.level)?;
    if check_rustc_version(&version_meta()?)? {
        info!("rustc version check successful");
        match check_audit("cargo audit --version") {
            Ok(success) => {
                if success {
                    info!("cargo audit version check successful");

                    // channels for thread comms
                    let (tx_stdout, rx_stdout) = channel();
                    let (tx_code, rx_code) = channel();

                    // start the threads
                    let deny_c = config.deny.clone();
                    let audit_handle = spawn(move || audit(&deny_c, tx_stdout, tx_code));
                    let rx_handle = spawn(move || receive_stdout(&rx_stdout));
                    let rx_code_handle = spawn(move || receive_code(&rx_code));

                    // wait for the thread to finish
                    audit_handle.join().map_err(handle_join_error)??;
                    rx_handle.join().map_err(handle_join_error)?;
                    let code = rx_code_handle.join().map_err(handle_join_error)?;
                    if code == 0 {
                        Ok(())
                    } else {
                        error!("Using token '{}'", config.token);
                        Err(AuditCheckError::AuditVersionCheck.into())
                    }
                } else {
                    Err(AuditCheckError::AuditVersionCheck.into())
                }
            }
            Err(e) => Err(e.context("cargo audit check has failed")),
        }
    } else {
        Err(AuditCheckError::RustcVersionCheck { msrv: MSRV }.into())
    }
}

fn receive_stdout(rx: &Receiver<String>) {
    while let Ok(message) = rx.recv() {
        info!("{message}");
    }
}

fn receive_code(rx: &Receiver<i32>) -> i32 {
    if let Ok(code) = rx.recv() {
        code
    } else {
        -1
    }
}
