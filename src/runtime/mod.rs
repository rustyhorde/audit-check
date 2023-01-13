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
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Version,
};
use rustc_version::version_meta;
use serde::{Deserialize, Serialize};
use std::{
    sync::mpsc::{channel, Receiver},
    thread::spawn,
};
use tokio::runtime::Runtime;
use tracing::{error, info, trace};

pub(crate) fn run() -> Result<()> {
    let config = Config::from_env()?;
    initialize(config.level)?;
    if check_rustc_version(&version_meta()?)? {
        trace!("rustc version check successful");
        match check_audit("cargo audit --version") {
            Ok(success) => {
                if success {
                    trace!("cargo audit version check successful");

                    // channels for thread comms
                    let (tx_stdout, rx_stdout) = channel();
                    let (tx_stderr, rx_stderr) = channel();
                    let (tx_code, rx_code) = channel();

                    // start the threads
                    let deny_c = config.deny.clone();
                    let audit_handle = spawn(move || audit(&deny_c, tx_stdout, tx_stderr, tx_code));
                    let stdout_handle = spawn(move || receive_stdout(&rx_stdout));
                    let stderr_handle = spawn(move || receive_stderr(&rx_stderr));
                    let code_handle = spawn(move || receive_code(&rx_code));

                    // wait for the thread to finish
                    audit_handle.join().map_err(handle_join_error)??;
                    let stdout_buf = stdout_handle.join().map_err(handle_join_error)?;
                    let stderr_buf = stderr_handle.join().map_err(handle_join_error)?;
                    let code = code_handle.join().map_err(handle_join_error)?;
                    if code == 0 {
                        Ok(())
                    } else if config.create_issue {
                        // Create the runtime
                        let rt = Runtime::new()?;
                        rt.block_on(async move {
                            match create_issue(config, stdout_buf, stderr_buf).await {
                                Ok(resp) => {
                                    info!("Issue {} created", resp.id);
                                }
                                Err(e) => error!("{e}"),
                            }
                        });
                        Err(AuditCheckError::RustSec.into())
                    } else {
                        Err(AuditCheckError::RustSec.into())
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

fn receive_stdout(rx: &Receiver<String>) -> Vec<String> {
    let mut buf = vec![];
    while let Ok(message) = rx.recv() {
        info!("{message}");
        buf.push(message);
    }
    buf
}

fn receive_stderr(rx: &Receiver<String>) -> Vec<String> {
    let mut buf = vec![];
    while let Ok(message) = rx.recv() {
        info!("{message}");
        buf.push(message);
    }
    buf
}

fn receive_code(rx: &Receiver<i32>) -> i32 {
    if let Ok(code) = rx.recv() {
        code
    } else {
        -1
    }
}

#[derive(Clone, Debug, Serialize)]
struct Issue {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    milestone: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignees: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Resp {
    id: usize,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

lazy_static! {
    static ref ID_REGEX: Regex = Regex::new(r#"ID: +(.*)"#).unwrap();
}

async fn create_issue(
    config: Config,
    stdout_buf: Vec<String>,
    stderr_buf: Vec<String>,
) -> Result<Resp> {
    let token = config.token;
    let owner_repo = config.owner_repo;
    let stdout = stdout_buf.join("\n");
    let id = if let Some(id_cap) = ID_REGEX.captures(&stdout) {
        id_cap.get(1).map_or("Unknown ID", |m| m.as_str())
    } else {
        "Unknown ID"
    };

    let mut headers = HeaderMap::new();
    let _old = headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );
    let _old = headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .build()?;

    let url = format!("https://api.github.com/repos/{owner_repo}/issues");
    info!("Posting to '{url}'");
    info!("STDOUT: {}", stdout_buf.join("\n"));
    info!("STDERR: {}", stderr_buf.join("\n"));
    let issue = Issue {
        title: id.to_string(),
        body: Some("This is another test issue".to_string()),
        milestone: None,
        labels: None,
        assignees: None,
    };
    let res = client
        .post(&url)
        .version(Version::HTTP_11)
        .bearer_auth(token)
        .json(&issue)
        .send()
        .await?;

    if res.status() == 201 {
        Ok(res.json::<Resp>().await?)
    } else {
        let body = res.bytes().await?;
        error!("{}", String::from_utf8_lossy(&body));
        Err(AuditCheckError::CreateIssue.into())
    }
}
