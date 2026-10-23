//! Loading the files of the speaker model entry into a ready labeller.

use super::{SegmentationModel, SpeakerDiarizer, SpeakerModel};
use crate::models::{ModelEntry, ModelStore};
use crate::{Error, Result};

/// The labeller the app and the CLI use.
pub type SpeakerLabeller = SpeakerDiarizer<SpeakerModel, SegmentationModel>;

/// Loads the voice model (first file of `entry`) and the turn model (second file).
pub fn load(store: &ModelStore, entry: &ModelEntry) -> Result<SpeakerLabeller> {
    let [voices, turns, ..] = entry.files.as_slice() else {
        return Err(Error::ModelMissing(entry.id.clone()));
    };
    Ok(SpeakerDiarizer::new(
        SpeakerModel::load(&store.file_path(entry, voices), &entry.id)?,
        SegmentationModel::load(&store.file_path(entry, turns), &entry.id)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Catalog;
    use crate::paths::Paths;

    #[test]
    fn a_model_that_is_not_downloaded_is_reported_as_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let store = ModelStore::new(&Paths::resolve(Some(dir.path().to_path_buf())));
        let catalog = Catalog::bundled().unwrap();
        let entry = catalog.find(crate::models::SPEAKER_MODEL_ID).unwrap();
        assert!(matches!(load(&store, entry), Err(Error::ModelMissing(_))));
    }
}
