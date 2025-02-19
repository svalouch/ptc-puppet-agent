// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2025 Stefan Valouch

// Models for de-serializing the various puppet files
use serde::Deserialize;

/// Contents of the lockfile
#[derive(Deserialize)]
pub(crate) struct Lockfile {
    pub disabled_message: String,
}

/// Contents of the `last_run_summary.yaml`
#[derive(Deserialize)]
pub(crate) struct LastRunSummary {
    pub version: SummaryVersion,
    pub application: SummaryApplication,
    pub resources: SummaryResources,
    pub time: SummaryTime,
    pub events: SummaryEvents,
}

#[derive(Deserialize)]
pub(crate) struct SummaryVersion {
    pub config: Option<String>,
    pub puppet: String,
}

#[derive(Deserialize)]
pub(crate) struct SummaryApplication {
    pub converged_environment: String,
}

#[derive(Deserialize)]
pub(crate) struct SummaryResources {
    pub failed: u32,
    pub failed_to_restart: u32,
    pub total: u32,
}

#[derive(Deserialize)]
pub(crate) struct SummaryTime {
    pub last_run: u64,
}

#[derive(Deserialize)]
pub(crate) struct SummaryEvents {
    pub failure: u32,
    pub noop: Option<u32>,
}

/// Contents of the `last_run_report.yaml`
#[derive(Deserialize)]
pub(crate) struct LastRunReport {
    pub status: String,
    pub cached_catalog_status: String,
}
