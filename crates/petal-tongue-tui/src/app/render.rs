// SPDX-License-Identifier: AGPL-3.0-or-later
//! TUI rendering logic

use crate::state::{TUIState, View};
use ratatui::Frame;

use crate::views::{
    render_dashboard, render_devices, render_livespore, render_logs, render_neural_api,
    render_nucleus, render_primals, render_topology,
};

/// Render the current view to the frame
pub(super) fn render_current_view(f: &mut Frame<'_>, state: &TUIState, view: View) {
    match view {
        View::Dashboard => render_dashboard(f, state),
        View::Topology => render_topology(f, state),
        View::Devices => render_devices(f, state),
        View::Primals => render_primals(f, state),
        View::Logs => render_logs(f, state),
        View::NeuralAPI => render_neural_api(f, state),
        View::Nucleus => render_nucleus(f, state),
        View::LiveSpore => render_livespore(f, state),
    }
}
