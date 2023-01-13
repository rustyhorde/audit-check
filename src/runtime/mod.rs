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
use anyhow::{anyhow, Result};
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
                    } else if config.create_issue {
                        // Create the runtime
                        info!("Creating Issue");
                        let rt = Runtime::new()?;
                        rt.block_on(async move {
                            match create_issue(config).await {
                                Ok(_) => info!("success"),
                                Err(e) => error!("{e}"),
                            }
                        });
                        Err(AuditCheckError::AuditVersionCheck.into())
                    } else {
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
    id: String,
}

async fn create_issue(config: Config) -> Result<()> {
    let token = config.token;
    let owner_repo = config.owner_repo;

    let mut headers = HeaderMap::new();
    let _old = headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );
    let _old = headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    let client = Client::builder().default_headers(headers).build()?;

    let url = format!("https://api.github.com/repos/{owner_repo}/issues");
    info!("Posting to '{url}'");
    let issue = Issue {
        title: "Test Issue 2".to_string(),
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
        let resp = res.json::<Resp>().await?;
        info!("Issue {} created", resp.id);
        Ok(())
    } else {
        error!("Response: {res:?}");
        let body = res.bytes().await?;
        error!("{}", String::from_utf8_lossy(&body));
        Err(anyhow!("create issue failed"))
    }
}
