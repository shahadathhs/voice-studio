/// How a command should talk to the terminal: `--json` switches stdout to machine-readable events,
/// `--quiet` silences progress and informational messages on stderr.
#[derive(Debug, Clone, Copy)]
pub struct Output {
    pub json: bool,
    pub quiet: bool,
}

impl Output {
    /// Human progress and messages go to stderr unless suppressed or replaced by JSON events.
    pub fn show_messages(self) -> bool {
        !self.json && !self.quiet
    }
}
