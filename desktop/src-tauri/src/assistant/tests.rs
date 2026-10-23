use super::*;

fn request(task: Task, language: Option<&str>, model: &str) -> Request {
    Request {
        item_id: 1,
        task,
        language: language.map(String::from),
        model: model.into(),
    }
}

fn paths() -> (tempfile::TempDir, Paths) {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::resolve(Some(dir.path().to_path_buf()));
    (dir, paths)
}

#[test]
fn a_translation_needs_a_language() {
    let (_dir, paths) = paths();
    let error = check(&paths, &request(Task::Translation, None, "qwen2.5-1.5b")).unwrap_err();
    assert!(matches!(error, Error::Input(_)), "{error}");
}

#[test]
fn unsupported_languages_are_rejected() {
    let (_dir, paths) = paths();
    let error = check(&paths, &request(Task::Summary, Some("xx"), "qwen2.5-1.5b")).unwrap_err();
    assert!(
        error.to_string().contains("unsupported language"),
        "{error}"
    );
}

#[test]
fn only_installed_language_models_can_be_used() {
    let (_dir, paths) = paths();
    let missing = check(&paths, &request(Task::Summary, None, "qwen2.5-1.5b")).unwrap_err();
    assert!(matches!(missing, Error::ModelMissing(_)), "{missing}");
    let wrong_kind = check(&paths, &request(Task::Summary, None, "whisper-base")).unwrap_err();
    assert!(matches!(wrong_kind, Error::Input(_)), "{wrong_kind}");
}

#[test]
fn the_outcome_is_reported_for_the_screen() {
    let done = finished(
        Task::Translation,
        Ok(Done {
            text: None,
            item_id: Some(7),
        }),
    );
    assert_eq!((done.outcome, done.item_id), ("done", Some(7)));
    assert_eq!(
        finished(Task::Summary, Err(Error::Cancelled)).outcome,
        "cancelled"
    );
    let failed = finished(Task::Summary, Err(Error::Engine("boom".into())));
    assert_eq!(
        (failed.outcome, failed.message.as_deref()),
        ("failed", Some("boom"))
    );
}
