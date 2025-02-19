// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2025 Stefan Valouch

// Prometheus handling

use prometheus::TextEncoder;
use std::path::Path;
use tracing::error;

#[derive(Debug)]
pub(crate) struct PromOutputError;

/// Encodes the metrics currently registered to the default-registry into a string.
pub(crate) fn encode() -> Result<String, PromOutputError> {
    TextEncoder::new()
        .encode_to_string(&prometheus::gather())
        .map_err(|e| {
            error!("Could not encode metrics: {e}");
            PromOutputError
        })
}

/// Writes the metrics currently registered to the default-registry to a file. The resulting file
/// is world-readable.
pub(crate) fn to_file<T>(path: T) -> Result<(), PromOutputError>
where
    T: AsRef<Path> + core::fmt::Debug + Send,
{
    crate::fs::write_file_atomic(encode()?.as_bytes(), path, 0o644).map_err(|e| {
        error!("Could not write metrics to file: {e}");
        PromOutputError
    })?;
    Ok(())
}

/// Prints the metrics currently registered to the default-registry to stdout.
pub fn to_stdout() {
    println!("{}", encode().unwrap());
}
