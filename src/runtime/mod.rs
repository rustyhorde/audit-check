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
    collections::BTreeMap,
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
    rx.recv().unwrap_or(-1)
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
    static ref CRATE_REGEX: Regex = Regex::new(r"Crate: +(.*)").expect("Invalid CRATE_REGEX");
    static ref VERSION_REGEX: Regex = Regex::new(r"Version: +(.*)").expect("Invalid VERSION_REGEX");
    static ref WARNING_REGEX: Regex = Regex::new(r"Warning: +(.*)").expect("Invalid WARNING_REGEX");
    static ref TITLE_REGEX: Regex = Regex::new(r"Title: +(.*)").expect("Invalid TITLE_REGEX");
    static ref DATE_REGEX: Regex = Regex::new(r"Date: +(.*)").expect("Invalid DATE_REGEX");
    static ref SOLUTION_REGEX: Regex =
        Regex::new(r"Solution: +(.*)").expect("Invalid SOLUTION_REGEX");
    static ref ID_REGEX: Regex = Regex::new(r"ID: +(RUSTSEC.*)").expect("Invalid ID_REGEX");
    static ref URL_REGEX: Regex = Regex::new(r"URL: +(https:.*)").expect("Invalid URL_REGEX");
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Rustsec {
    id: String,
    url: String,
    krate: String,
    version: String,
    warning: String,
    title: String,
    date: String,
    solution: String,
}

async fn create_issue(
    config: Config,
    stdout_buf: Vec<String>,
    _stderr_buf: Vec<String>,
) -> Result<Resp> {
    let token = config.token;
    let owner_repo = config.owner_repo;
    let stdout = stdout_buf.join("\n");
    let rustsec_map = parse(&stdout);
    let title = generate_title(&rustsec_map);
    let body = generate_body(&rustsec_map);

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
    let issue = Issue {
        title,
        body: Some(body),
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

fn parse(output: &str) -> BTreeMap<String, (String, Rustsec)> {
    let mut splits: Vec<String> = output.split("\n\n").map(str::to_string).collect();

    splits[0] = splits[0].lines().skip(4).collect::<Vec<&str>>().join("\n");

    splits
        .iter()
        .map(|x| (x, parse_rustsec(x)))
        .map(|(s, x)| (x.id.clone(), ((*s).to_string(), x)))
        .collect()
}

fn parse_rustsec(rustsec_str: &str) -> Rustsec {
    let id = parse_caps(&ID_REGEX, rustsec_str, "No ID");
    let url = parse_caps(&URL_REGEX, rustsec_str, "No URL");
    let krate = parse_caps(&CRATE_REGEX, rustsec_str, "No Crate");
    let version = parse_caps(&VERSION_REGEX, rustsec_str, "No Version");
    let warning = parse_caps(&WARNING_REGEX, rustsec_str, "No Warning");
    let title = parse_caps(&TITLE_REGEX, rustsec_str, "No Title");
    let date = parse_caps(&DATE_REGEX, rustsec_str, "No Date");
    let solution = parse_caps(&SOLUTION_REGEX, rustsec_str, "No Solution");

    Rustsec {
        id,
        url,
        krate,
        version,
        warning,
        title,
        date,
        solution,
    }
}

fn parse_caps(regex: &Regex, rustsec_str: &str, default: &str) -> String {
    regex
        .captures(rustsec_str)
        .map_or_else(
            || default,
            |caps| caps.get(1).map_or(default, |m| m.as_str()),
        )
        .to_string()
}

fn generate_title(rustsec_map: &BTreeMap<String, (String, Rustsec)>) -> String {
    rustsec_map.keys().fold(String::new(), |acc, key| {
        if acc.is_empty() {
            acc + key
        } else {
            acc + ", " + key
        }
    })
}

fn generate_body(rustsec_map: &BTreeMap<String, (String, Rustsec)>) -> String {
    rustsec_map.iter().fold(String::new(), |acc, (k, v)| {
        acc + &format!("# ‼️ {} ‼️\n{}\n\n````\n{}\n````\n\n", k, v.1.url, v.0)
    })
}

#[cfg(test)]
mod test {
    use super::{generate_title, parse};

    const TEST_RUSTSEC: &str = r"Crate:     aovec
Version:   1.1.0
Title:     Aovec<T> lacks bound on its Send and Sync traits allowing data races
Date:      2020-12-10
ID:        RUSTSEC-2020-0099
URL:       https://rustsec.org/advisories/RUSTSEC-2020-0099
Solution:  No fixed upgrade is available!
Dependency tree:
aovec 1.1.0
└── audit-check-test 0.1.0

Crate:     owning_ref
Version:   0.3.3
Title:     Multiple soundness issues in `owning_ref`
Date:      2022-01-26
ID:        RUSTSEC-2022-0040
URL:       https://rustsec.org/advisories/RUSTSEC-2022-0040
Solution:  No fixed upgrade is available!
Dependency tree:
owning_ref 0.3.3
└── parking_lot 0.4.8
    └── aovec 1.1.0
        └── audit-check-test 0.1.0

Crate:     anymap
Version:   0.12.1
Warning:   unmaintained
Title:     anymap is unmaintained.
Date:      2021-05-07
ID:        RUSTSEC-2021-0065
URL:       https://rustsec.org/advisories/RUSTSEC-2021-0065
Dependency tree:
anymap 0.12.1
└── audit-check-test 0.1.0

Crate:     smallvec
Version:   0.4.5
Warning:   unsound
Title:     smallvec creates uninitialized value of any type
Date:      2018-09-25
ID:        RUSTSEC-2018-0018
URL:       https://rustsec.org/advisories/RUSTSEC-2018-0018
Dependency tree:
smallvec 0.4.5
└── aovec 1.1.0
    └── audit-check-test 0.1.0
";

    #[test]
    fn parse_works() {
        assert_eq!(4, parse(TEST_RUSTSEC).len());
    }

    #[test]
    fn generate_title_works() {
        let rustsec_map = parse(TEST_RUSTSEC);
        assert_eq!(
            "RUSTSEC-2018-0018, RUSTSEC-2020-0099, RUSTSEC-2021-0065, RUSTSEC-2022-0040",
            generate_title(&rustsec_map)
        );
    }
}
