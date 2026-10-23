//! ONNX Runtime, shared by every model that uses it (speech and speaker identification).
//!
//! The runtime allows one environment per process, so it is created once here instead of by each model.

use std::path::Path;
use std::sync::OnceLock;

use ort::environment::Environment;
use ort::session::Session;

static ENVIRONMENT: OnceLock<Result<Environment, String>> = OnceLock::new();

/// Loads the ONNX model at `path` into a new session. The error is the runtime's own message; callers
/// add which model failed.
pub fn load_session(path: &Path) -> Result<Session, String> {
    let environment = ENVIRONMENT
        .get_or_init(|| {
            ort::environment::init()
                .with_name("voice-studio")
                .build()
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)?;
    Session::builder(environment)
        .and_then(|mut builder| builder.commit_from_file(path))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_not_a_model_is_an_error_and_loading_twice_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.onnx");
        std::fs::write(&path, b"not onnx").unwrap();
        assert!(load_session(&path).is_err());
        assert!(load_session(&path).is_err());
    }
}
