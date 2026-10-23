//! Clustering the voiceprints of a recording into speakers.

use super::cluster::{self, Stop};
use super::units::Unit;

/// A voiceprint made from at least this many samples (2 s) of speech can start a speaker.
const RELIABLE_SAMPLES: usize = 32_000;

/// Clusters the voiceprints made from enough speech (all of them when none is); the rest join
/// the nearest cluster, since a short stretch is too unreliable to start a speaker of its own.
pub fn group(voiceprints: &[Vec<f32>], units: &[Unit], stop: Stop) -> Vec<usize> {
    let mut reliable: Vec<usize> = (0..units.len())
        .filter(|i| units[*i].listening() >= RELIABLE_SAMPLES)
        .collect();
    if reliable.is_empty() {
        reliable = (0..units.len()).collect();
    }
    let core: Vec<Vec<f32>> = reliable.iter().map(|i| voiceprints[*i].clone()).collect();
    let core_labels = cluster::agglomerate(&core, stop);
    let rest: Vec<usize> = (0..units.len()).filter(|i| !reliable.contains(i)).collect();
    let others: Vec<&Vec<f32>> = rest.iter().map(|i| &voiceprints[*i]).collect();
    let mut labels = vec![0; units.len()];
    for (i, label) in reliable.iter().zip(&core_labels) {
        labels[*i] = *label;
    }
    for (i, label) in rest
        .iter()
        .zip(cluster::nearest_clusters(&core, &core_labels, &others))
    {
        labels[*i] = label;
    }
    labels
}
