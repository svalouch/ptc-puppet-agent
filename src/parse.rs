// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2025 Stefan Valouch

use serde::de::DeserializeOwned;
use std::{fs::File, path::Path};

fn open_file<T>(path: T) -> Result<File, String>
where
    T: AsRef<Path> + std::fmt::Debug + Send,
{
    let path = path.as_ref();
    if !path.exists() {
        return Err(format!("File {} does not exist", path.to_string_lossy()));
    }
    if !path.is_file() {
        return Err(format!("Path {} is not a file", path.to_string_lossy()));
    }
    File::open(path).map_err(|e| {
        format!(
            "Could not open file {} for reading: {e}",
            path.to_string_lossy()
        )
    })
}

/// Parse a file as YAML and return the result, or a textual error message
pub(crate) fn parse_file_yaml<T, P>(path: T) -> Result<P, String>
where
    T: AsRef<Path> + std::fmt::Debug + Send,
    P: DeserializeOwned,
{
    let path = path.as_ref();
    let f = open_file(path)?;
    serde_yaml::from_reader(f).map_err(|e| {
        format!(
            "Failure in YAML deserialization of {}: {e}",
            path.to_string_lossy()
        )
    })
}

// /// Parse a file as JSON and return the result, or a textual error message
// pub(crate) fn parse_file_json<T, P>(path: T) -> Result<P, String>
// where
//     T: AsRef<Path> + std::fmt::Debug + Send,
//     P: DeserializeOwned,
// {
//     let path = path.as_ref();
//     serde_json::from_reader(open_file(path)?).map_err(|e| {
//         format!(
//             "Failure in JSON deserialization of {}: {e}",
//             path.to_string_lossy()
//         )
//     })
// }
