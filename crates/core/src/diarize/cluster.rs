//! Average-linkage agglomerative clustering on cosine distance, as the earlier prototype did with
//! scikit-learn. Every embedding starts as its own cluster and the two closest clusters are merged
//! until the wanted number of clusters is reached or the closest pair is too far apart.

/// When to stop merging.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stop {
    /// Stop at exactly this many clusters (or fewer if there are fewer embeddings).
    Clusters(usize),
    /// Stop when the closest two clusters are further apart than this cosine distance.
    Distance(f32),
}

fn normalized(embedding: &[f32]) -> Vec<f32> {
    let length = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    let scale = if length > 0.0 { 1.0 / length } else { 0.0 };
    embedding.iter().map(|x| x * scale).collect()
}

fn distance_matrix(embeddings: &[Vec<f32>]) -> Vec<f32> {
    let n = embeddings.len();
    let unit: Vec<Vec<f32>> = embeddings.iter().map(|e| normalized(e)).collect();
    let mut matrix = vec![0.0; n * n];
    for i in 0..n {
        for j in (i + 1)..n {
            let dot: f32 = unit[i].iter().zip(&unit[j]).map(|(a, b)| a * b).sum();
            matrix[i * n + j] = 1.0 - dot;
            matrix[j * n + i] = 1.0 - dot;
        }
    }
    matrix
}

struct State {
    n: usize,
    distance: Vec<f32>,
    size: Vec<f32>,
    active: Vec<bool>,
    /// Nearest other active cluster of each cluster, and how far away it is.
    nearest: Vec<(usize, f32)>,
}

impl State {
    fn new(embeddings: &[Vec<f32>]) -> State {
        let n = embeddings.len();
        let mut state = State {
            n,
            distance: distance_matrix(embeddings),
            size: vec![1.0; n],
            active: vec![true; n],
            nearest: vec![(0, f32::INFINITY); n],
        };
        (0..n).for_each(|i| state.refresh(i));
        state
    }

    fn refresh(&mut self, i: usize) {
        self.nearest[i] = (0..self.n)
            .filter(|j| *j != i && self.active[*j])
            .map(|j| (j, self.distance[i * self.n + j]))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((i, f32::INFINITY));
    }

    /// The closest pair `(i, j, distance)` among active clusters.
    fn closest(&self) -> Option<(usize, usize, f32)> {
        (0..self.n)
            .filter(|i| self.active[*i])
            .map(|i| (i, self.nearest[i].0, self.nearest[i].1))
            .filter(|(_, _, d)| d.is_finite())
            .min_by(|a, b| a.2.total_cmp(&b.2))
    }

    /// Merges `j` into `i`; the distance to every other cluster becomes the size-weighted average.
    fn merge(&mut self, i: usize, j: usize) {
        let (si, sj) = (self.size[i], self.size[j]);
        for m in 0..self.n {
            if m == i || m == j || !self.active[m] {
                continue;
            }
            let merged = (si * self.distance[i * self.n + m] + sj * self.distance[j * self.n + m])
                / (si + sj);
            self.distance[i * self.n + m] = merged;
            self.distance[m * self.n + i] = merged;
        }
        self.size[i] = si + sj;
        self.active[j] = false;
        self.refresh(i);
        for m in 0..self.n {
            if self.active[m] && m != i && (self.nearest[m].0 == i || self.nearest[m].0 == j) {
                self.refresh(m);
            }
        }
    }
}

/// The full merge history, closest pair first. With average linkage the distances never decrease,
/// so stopping early is the same as keeping a prefix of this list.
fn dendrogram(embeddings: &[Vec<f32>]) -> Vec<(usize, usize, f32)> {
    let mut state = State::new(embeddings);
    let mut merges = Vec::with_capacity(embeddings.len().saturating_sub(1));
    while let Some((i, j, distance)) = state.closest() {
        state.merge(i, j);
        merges.push((i, j, distance));
    }
    merges
}

/// How many merges to apply for `stop`, given the merge distances in ascending order.
fn merges_to_apply(heights: &[f32], stop: Stop) -> usize {
    match stop {
        Stop::Clusters(wanted) => (heights.len() + 1).saturating_sub(wanted.max(1)),
        Stop::Distance(limit) => heights.partition_point(|d| *d <= limit),
    }
}

/// Clusters `embeddings` and returns one label per embedding, numbered by first appearance
/// (the first embedding is always in cluster 0).
pub fn agglomerate(embeddings: &[Vec<f32>], stop: Stop) -> Vec<usize> {
    let merges = dendrogram(embeddings);
    let heights: Vec<f32> = merges.iter().map(|m| m.2).collect();
    let mut owner: Vec<usize> = (0..embeddings.len()).collect();
    for (i, j, _) in merges.iter().take(merges_to_apply(&heights, stop)) {
        owner.iter_mut().filter(|o| **o == *j).for_each(|o| *o = *i);
    }
    renumber(&owner)
}

/// Relabels so numbers go 0, 1, 2... in order of first appearance.
pub fn renumber(labels: &[usize]) -> Vec<usize> {
    let mut seen: Vec<usize> = Vec::new();
    labels
        .iter()
        .map(|label| {
            seen.iter().position(|s| s == label).unwrap_or_else(|| {
                seen.push(*label);
                seen.len() - 1
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Directions in the plane: cosine distance depends only on the angle.
    fn at(degrees: f32) -> Vec<f32> {
        let radians = degrees.to_radians();
        vec![radians.cos(), radians.sin()]
    }

    fn two_groups() -> Vec<Vec<f32>> {
        vec![at(0.0), at(5.0), at(90.0), at(3.0), at(95.0), at(88.0)]
    }

    #[test]
    fn a_fixed_cluster_count_splits_the_two_groups() {
        assert_eq!(
            agglomerate(&two_groups(), Stop::Clusters(2)),
            [0, 0, 1, 0, 1, 1]
        );
    }

    #[test]
    fn a_distance_limit_finds_the_groups_without_being_told_how_many() {
        // Within a group the distance is about 0.004; between groups it is about 1.
        assert_eq!(
            agglomerate(&two_groups(), Stop::Distance(0.5)),
            [0, 0, 1, 0, 1, 1]
        );
        assert_eq!(agglomerate(&two_groups(), Stop::Distance(0.0001)).len(), 6);
        let loose = agglomerate(&two_groups(), Stop::Distance(1.5));
        assert!(loose.iter().all(|l| *l == 0));
    }

    #[test]
    fn a_tight_limit_keeps_every_embedding_apart() {
        let labels = agglomerate(&two_groups(), Stop::Distance(0.0));
        assert_eq!(labels, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn asking_for_more_clusters_than_embeddings_changes_nothing() {
        assert_eq!(agglomerate(&[at(0.0), at(90.0)], Stop::Clusters(5)), [0, 1]);
        assert_eq!(agglomerate(&[], Stop::Clusters(2)).len(), 0);
        assert_eq!(agglomerate(&[at(0.0)], Stop::Distance(0.3)), [0]);
    }

    #[test]
    fn labels_are_numbered_by_first_appearance() {
        assert_eq!(renumber(&[7, 7, 3, 7, 9, 3]), [0, 0, 1, 0, 2, 1]);
    }
}
