// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

/// The error type
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The path is not a directory
    #[error("path is not a directory: {0}")]
    NotDir(PathBuf),
}
