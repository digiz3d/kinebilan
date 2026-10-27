use std::sync::mpsc::{self, Receiver};
use std::thread;

use self_update::update::ReleaseStatus;

const REPO_OWNER: &str = "digiz3d";
const REPO_NAME: &str = "kinebilan";
const TAG_PREFIX: &str = "kinebilan-";
const BIN_NAME: &str = "kinebilan";

#[cfg(windows)]
const ASSET_IDENTIFIER: &str = "windows-x64.zip";
#[cfg(windows)]
const BIN_PATH_IN_ARCHIVE: &str = "kinebilan.exe";

#[cfg(target_os = "macos")]
const ASSET_IDENTIFIER: &str = "macos-universal.tar.gz";
#[cfg(target_os = "macos")]
const BUNDLE_PATH_IN_ARCHIVE: &str = "Kinebilan.app";

/// Checks GitHub for a newer release in the background and installs it.
/// The receiver yields the installed version once an update has been applied.
pub fn spawn_check() -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    if cfg!(debug_assertions) {
        return receiver;
    }
    thread::spawn(move || match install_latest() {
        Ok(Some(version)) => {
            let _ = sender.send(version);
        }
        Ok(None) => {}
        Err(error) => eprintln!("Mise à jour impossible : {error}"),
    });
    receiver
}

fn install_latest() -> Result<Option<String>, self_update::errors::Error> {
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .tag_prefix(TAG_PREFIX)
        .bin_name(BIN_NAME)
        .asset_identifier(ASSET_IDENTIFIER)
        .current_version(env!("CARGO_PKG_VERSION"))
        .unattended();
    #[cfg(windows)]
    builder.bin_path_in_archive(BIN_PATH_IN_ARCHIVE);
    #[cfg(target_os = "macos")]
    builder.bundle_path_in_archive(BUNDLE_PATH_IN_ARCHIVE);
    let status = builder.build()?.update_extended()?;
    Ok(match status {
        ReleaseStatus::Updated(release) => Some(release.version().to_string()),
        _ => None,
    })
}
