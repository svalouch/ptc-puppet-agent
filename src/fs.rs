// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2025 Stefan Valouch

use std::{
    fs::Permissions,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use tempfile::NamedTempFile;
use tracing::trace;

/// Writes the content to a file in an atomic way. The directory pointed at by `path` must exist
/// prior to calling this function. `mode` is a unix permission mode, such as `0o644`.
pub(crate) fn write_file_atomic<T>(content: &[u8], path: T, mode: u32) -> io::Result<()>
where
    T: AsRef<Path> + core::fmt::Debug + Send,
{
    let mut dir = path.as_ref().to_owned();
    dir.pop();

    trace!("Creating temporary file in {path:?}");
    let tmp_file = NamedTempFile::new_in(dir).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("creating the temp file failed: {e}"),
        )
    })?;
    let (mut tmp_file, tmp_path) = tmp_file.into_parts();
    trace!("Temporary file created as {tmp_path:?}");
    tmp_file.set_permissions(Permissions::from_mode(mode))?;
    tmp_file.write_all(content)?;
    tmp_path.persist(path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("persisting the temporary file failed: {e}"),
        )
    })
}
