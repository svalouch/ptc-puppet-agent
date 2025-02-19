// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2025 Stefan Valouch

use clap::Parser;
use prometheus::{
    core::{AtomicU64, GenericGauge},
    register_int_gauge, register_int_gauge_vec,
};
use regex::Regex;
use std::{path::PathBuf, process};
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;

mod fs;
mod models;
mod parse;
mod prom;

// The following defaults are for:
// - Standard Debian prometheus-node-exporter textfile-collector location
// - Official PuppetLabs puppet-agent

/// Default textfile-collector output file if not overwritten.
const DEFAULT_OUTPUT_FILE: &str = "/var/lib/prometheus/node-exporter/puppet-agent.prom";
/// Default "disabled" lockfile location.
const DEFAULT_DISABLED_LOCKFILE: &str = "/opt/puppetlabs/puppet/cache/state/agent_disabled.lock";
/// Default summary file location.
const DEFAULT_LASTRUNFILE: &str = "/opt/puppetlabs/puppet/public/last_run_summary.yaml";
/// Default run report location.
const DEFAULT_LASTRUNREPORT: &str = "/opt/puppetlabs/puppet/cache/state/last_run_report.yaml";

/// Prometheus Textfile Collector – Puppet Agent
///
/// Exports whether puppet-agent has been disabled locally (via e.g. "puppet agent --disable
/// [message]") and some metrics about the most recent run by examining the last run summary and
/// run report files.
///
/// To debug it, set the environment variable "PTC_LOG" to "debug" to make its output more verbose.
/// Use the "--stdout" switch to make it print instead of writing to the filesystem.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Path to the file where the output will be written to (unless --stdout is set).
    #[arg(long, short, env, default_value = DEFAULT_OUTPUT_FILE)]
    output_file: PathBuf,
    /// Agent disable lockfile, its existence indicates whether the agent has been --disabled. The
    /// optional disabled-message is exported unless --no-disabled-message is set.
    #[arg(long, short, env, default_value = DEFAULT_DISABLED_LOCKFILE)]
    agent_disabled_lockfile: PathBuf,
    /// Last run summary with metrics on the success of the most recent run.
    #[arg(long, short='r', env, default_value = DEFAULT_LASTRUNFILE)]
    lastrunfile: PathBuf,
    /// Last run report with additional metrics for cached catalog status.
    #[arg(long, short, env, default_value = DEFAULT_LASTRUNREPORT)]
    lastrunreport: PathBuf,
    /// Do not include the disabled-message as label value.
    #[arg(long, short, env)]
    no_disabled_message: bool,
    /// Print to stdout instead of the output-file.
    #[arg(long, short)]
    stdout: bool,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("PTC_LOG"))
        .init();
    let args = Cli::parse();

    debug!("Output file is set to {:?}", args.output_file);
    debug!("'lastrunfile' is set to {:?}", args.lastrunfile);
    debug!("'lastrunreport' is set to {:?}", args.lastrunreport);
    debug!(
        "'agent_disabled_lockfile' is set to {:?}",
        args.agent_disabled_lockfile
    );

    process_agent_disabled_lockfile(&args.agent_disabled_lockfile, !args.no_disabled_message);
    let mut success = match process_files(&args.lastrunfile, &args.lastrunreport) {
        Ok(()) => true,
        Err(e) => {
            error!("{e}");
            false
        }
    };

    let g = register_int_gauge!(
        "puppet_agent_collector_success",
        "Indicates whether the collector was successful or encountered problems"
    )
    .unwrap();
    g.set(success.into());

    if args.stdout {
        prom::to_stdout();
    } else {
        success &= prom::to_file(args.output_file).is_ok();
    }

    process::exit(match success {
        true => 0,
        false => 1,
    });
}

/// Check for the agent's lockfile
fn process_agent_disabled_lockfile(lockfile: &PathBuf, include_message: bool) {
    let (exists, msg) = match std::fs::read_to_string(lockfile) {
        Ok(data) => {
            debug!("Lockfile could be opened for reading, agent is disabled");
            let lock_msg: String = if include_message {
                match serde_json::from_str::<models::Lockfile>(&data) {
                    Ok(msg) => msg
                        .disabled_message
                        .trim()
                        .chars()
                        .filter(|c| !c.is_control() && c != &'"')
                        .collect(),
                    Err(e) => {
                        warn!("Unable to parse lockfile contents: {e}");
                        "__ERR__ failed to decode agent disabled message".into()
                    }
                }
            } else {
                "".into()
            };
            (true, Some(lock_msg))
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                debug!("Lockfile does not exist, agent is not disabled");
                (false, None)
            }
            std::io::ErrorKind::PermissionDenied => {
                warn!("Permission denied reading the agent disabled lockfile");
                (false, None)
            }
            other => {
                warn!("Unexpected error when opening agent lockfile: {other}");
                // grab the admin's attention by pretending it exists
                (true, Some(String::from("__ERR__")))
            }
        },
    };
    debug!("Agent is disabled: {exists}");

    let is_enabled = if exists { 0 } else { 1 };
    if include_message {
        register_int_gauge_vec!(
            "puppet_agent_enabled",
            "Whether the agent is enabled (allowed to run)",
            &["message"]
        )
        .unwrap()
        .with_label_values(&[&msg.unwrap_or("".into())])
        .set(is_enabled);
    } else {
        register_int_gauge!(
            "puppet_agent_enabled",
            "Whether the agent is enabled (allowed to run)",
        )
        .unwrap()
        .set(is_enabled);
    }
}

/// Reads the agent's output files and generates metrics from them
fn process_files(lastrunfile: &PathBuf, lastrunreport: &PathBuf) -> Result<(), String> {
    debug!("Parsing lastrunfile: {}", &lastrunfile.to_string_lossy());
    let summary: models::LastRunSummary = parse::parse_file_yaml(lastrunfile).map_err(|e| {
        format!(
            "Parsing lastrunfile {} failed: {e}",
            lastrunfile.to_string_lossy()
        )
    })?;

    debug!("Parsing lastrunreport: {}", lastrunreport.to_string_lossy());
    let report: models::LastRunReport = parse::parse_file_yaml(lastrunreport).map_err(|e| {
        format!(
            "Parsing lastrunfile {} failed: {e}",
            lastrunreport.to_string_lossy()
        )
    })?;

    // There are multiple ways the last run summary either directly (in the case of missing config)
    // or indirectly (in case of failures during the run) tells us that the run failed. This aims
    // to catch them all in one go:
    let failed = report.cached_catalog_status == "on_failure"
        || report.status == "failed"
        || summary.version.config.is_none()
        || (summary.resources.failed
            + summary.resources.failed_to_restart
            + summary.events.failure)
            != 0;
    register_int_gauge!(
        "puppet_agent_run_successful",
        "Whether the most recent run was a success"
    )
    .unwrap()
    .set((!failed).into());

    let version_regex = Regex::new(r#"^(\d+)\.(\d+)\.(\d+)$"#).unwrap();
    let agent_version = if !version_regex.is_match(&summary.version.puppet) {
        warn!(
            "Failed to validate the agent version output: {} (using dummy)",
            &summary.version.puppet
        );
        "0.0.0"
    } else {
        &summary.version.puppet
    };
    register_int_gauge_vec!(
        "puppet_agent_info",
        "Information about the agent and the environment",
        &["version", "environment"]
    )
    .unwrap()
    .with_label_values(&[agent_version, &summary.application.converged_environment])
    .set(1);

    // they haven't written a macro for u64. Normally, we wouldn't bother, but timestamps should
    // really be unsigned.
    let run_timestamp = {
        let run_timestamp = GenericGauge::<AtomicU64>::new(
            "puppet_agent_run_time",
            "Timestamp of the most recent puppet run",
        )
        .unwrap();
        prometheus::register(Box::new(run_timestamp.clone()))
            .map(|()| run_timestamp)
            .unwrap()
    };
    run_timestamp.set(summary.time.last_run);

    register_int_gauge!(
        "puppet_agent_run_resources_failed",
        "Amount of failed resources in the most recent run"
    )
    .unwrap()
    .set(summary.resources.failed as i64);

    register_int_gauge!(
        "puppet_agent_run_resources_failed_to_restart",
        "Amount of resources in the most recent run that failed their restart-action"
    )
    .unwrap()
    .set(summary.resources.failed_to_restart as i64);

    register_int_gauge!(
        "puppet_agent_run_resources_total",
        "Total number of resources in the most recent run"
    )
    .unwrap()
    .set(summary.resources.total as i64);

    register_int_gauge!(
        "puppet_agent_run_events_noop",
        "Number of events if the most recent run was a NOOP run"
    )
    .unwrap()
    .set(summary.events.noop.unwrap_or(0) as i64);

    register_int_gauge!(
        "puppet_agent_run_events_failed",
        "Number of events in the most recent run that failed"
    )
    .unwrap()
    .set(summary.events.failure as i64);

    Ok(())
}
