// Copyright (c) 2023 audit-check developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::Result;
use std::{
    io::{BufRead, BufReader},
    process::{ChildStderr, ChildStdout, ExitStatus, Stdio},
    sync::mpsc::Sender,
    thread,
    time::Duration,
};
use tracing::trace;

use crate::{error::AuditCheckError, utils::handle_join_error};

pub(crate) fn audit(
    deny: &str,
    tx_stdout: Sender<String>,
    tx_stderr: Sender<String>,
    tx_code: Sender<i32>,
) -> Result<()> {
    let command = format!("cargo audit -D{deny}");
    trace!("Running '{command}'");
    let mut cmd = std::process::Command::new("sh");
    let _ = cmd.arg("-c");
    let _ = cmd.arg(command);
    let _ = cmd.stdout(Stdio::piped());
    let _ = cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().ok_or(AuditCheckError::Stdout)?;
    let stdout_handle = thread::spawn(move || handle_stdout(stdout, &tx_stdout));

    let stderr = child.stderr.take().ok_or(AuditCheckError::Stderr)?;
    let stderr_handle = thread::spawn(move || handle_stderr(stderr, &tx_stderr));

    loop {
        match child.try_wait() {
            Ok(Some(exit_status)) => {
                handle_status(exit_status, tx_code)?;
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(500)),
            Err(e) => return Err(e.into()),
        }
    }

    stdout_handle.join().map_err(handle_join_error)??;
    stderr_handle.join().map_err(handle_join_error)??;
    Ok(())
}

fn handle_stdout(stdout: ChildStdout, tx: &Sender<String>) -> Result<()> {
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        tx.send(line)?;
    }
    Ok(())
}

fn handle_stderr(stderr: ChildStderr, tx: &Sender<String>) -> Result<()> {
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines().map_while(Result::ok) {
        tx.send(line)?;
    }
    Ok(())
}

fn handle_status(exit_status: ExitStatus, tx: Sender<i32>) -> Result<()> {
    let code = exit_status.code().ok_or(AuditCheckError::Code)?;
    tx.send(code)?;
    drop(tx);
    Ok(())
}
