//! Reading a voice's style table and blending several of them.

use std::path::Path;

use crate::{Error, Result};

/// Numbers in one style vector.
pub(in crate::tts) const STYLE_SIZE: usize = 256;

/// Reads a voice file: little-endian f32 numbers, one 256-number row per sentence length.
pub(in crate::tts) fn read_table(path: &Path) -> Result<Vec<f32>> {
    table_from_bytes(&std::fs::read(path)?, &path.display().to_string())
}

fn table_from_bytes(bytes: &[u8], label: &str) -> Result<Vec<f32>> {
    let (words, _) = bytes.as_chunks::<4>();
    let numbers: Vec<f32> = words.iter().map(|b| f32::from_le_bytes(*b)).collect();
    if numbers.is_empty() || !numbers.len().is_multiple_of(STYLE_SIZE) {
        return Err(Error::Engine(format!(
            "kokoro: voice file {label} is damaged"
        )));
    }
    Ok(numbers)
}

/// The weighted average of tables, row by row. Weights are used as given (they sum to 1 in a
/// preset); the tables must have the same number of rows.
pub(in crate::tts) fn blend(tables: &[(Vec<f32>, f32)]) -> Result<Vec<f32>> {
    let Some((first, _)) = tables.first() else {
        return Err(Error::Input("there is nothing to blend".into()));
    };
    if tables.iter().any(|(table, _)| table.len() != first.len()) {
        return Err(Error::Input(
            "these voices cannot be blended: their style tables differ in size".into(),
        ));
    }
    let mut mixed = vec![0.0_f32; first.len()];
    for (table, weight) in tables {
        for (out, value) in mixed.iter_mut().zip(table) {
            *out += value * weight;
        }
    }
    Ok(mixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blend_is_the_weighted_average_row_by_row() {
        let a = vec![1.0; STYLE_SIZE * 2];
        let b = vec![3.0; STYLE_SIZE * 2];
        let mixed = blend(&[(a, 0.75), (b, 0.25)]).unwrap();
        assert_eq!(mixed.len(), STYLE_SIZE * 2);
        assert!(mixed.iter().all(|v| (v - 1.5).abs() < 1e-6));
    }

    #[test]
    fn tables_of_different_sizes_cannot_be_blended() {
        let short = vec![0.0; STYLE_SIZE];
        let long = vec![0.0; STYLE_SIZE * 2];
        assert!(blend(&[(short, 0.5), (long, 0.5)]).is_err());
        assert!(blend(&[]).is_err());
    }

    #[test]
    fn tables_are_little_endian_rows_of_256_numbers() {
        let bytes: Vec<u8> = (0..STYLE_SIZE)
            .flat_map(|_| 2.5_f32.to_le_bytes())
            .collect();
        let table = table_from_bytes(&bytes, "x").unwrap();
        assert_eq!((table.len(), table[7]), (STYLE_SIZE, 2.5));
        assert!(table_from_bytes(&bytes[..10], "x").is_err());
        assert!(table_from_bytes(&[], "x").is_err());
    }
}
