// SPDX-License-Identifier: AGPL-3.0-or-later
//! `petaltongue ide` — the Gaia IDE in a window of its own.
//!
//! The window is a face for the `gaia-ide` server already running on this
//! device (biomeOS). It registers nothing with the ecosystem and starts no
//! service: it asks that one server, on the loopback address, with the
//! session token that server left for this user.

use crate::error::AppError;

/// A path as the server wants it: whole, from the root. A path given
/// relative to where the command was run is made whole from there.
pub fn whole_path(given: Option<String>) -> Option<String> {
    let given = given.filter(|path| !path.trim().is_empty())?;
    Some(std::path::absolute(&given).map_or(given, |whole| whole.to_string_lossy().into_owned()))
}

/// Open the window on the calling thread, which must be the main thread
/// (winit requires it), and stay until it is closed.
pub fn run_on_main_thread(
    port: Option<u16>,
    workspace: Option<String>,
    file: Option<String>,
) -> Result<(), AppError> {
    use petal_tongue_ui::ide;

    let options = ide::IdeOptions {
        port,
        workspace: whole_path(workspace),
        file: whole_path(file),
    };
    tracing::info!(?options, "Opening the Gaia IDE window");
    let native = crate::ui_mode::native_options_with_any_thread(ide::viewport());
    ide::run(options, native).map_err(AppError::Eframe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_path_is_left_as_it_is() {
        assert_eq!(
            whole_path(Some("/work/a.rs".to_owned())).as_deref(),
            Some("/work/a.rs")
        );
    }

    #[test]
    fn a_path_from_here_is_made_whole() {
        let here = std::env::current_dir().expect("the tests run somewhere");
        assert_eq!(
            whole_path(Some("src/a.rs".to_owned())),
            Some(here.join("src/a.rs").to_string_lossy().into_owned())
        );
    }

    #[test]
    fn no_path_and_an_empty_path_are_no_path() {
        assert_eq!(whole_path(None), None);
        assert_eq!(whole_path(Some("  ".to_owned())), None);
    }
}
