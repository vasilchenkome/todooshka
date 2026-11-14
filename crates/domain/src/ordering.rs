//! Ordering rules inside a column.
//!
//! Positions are fractional numbers spaced by 1.0; an insert between neighbours takes the
//! midpoint. When the gap gets too small, `between` returns `None` and the move use case
//! renumbers the column with [`evenly`].

/// Step for an empty column and for inserts at the top or bottom.
pub const STEP: f64 = 1.0;

/// The smallest gap that can still be halved without losing precision.
const MIN_GAP: f64 = 1e-9;

/// Position for an insert between neighbours. `None` means the gap is exhausted.
pub fn between(before: Option<f64>, after: Option<f64>) -> Option<f64> {
    match (before, after) {
        (None, None) => Some(STEP),
        (Some(before), None) => Some(before + STEP),
        (None, Some(after)) => Some(after - STEP),
        (Some(before), Some(after)) => {
            let gap = after - before;
            let middle = before + gap / 2.0;
            if gap > MIN_GAP && middle > before && middle < after {
                Some(middle)
            } else {
                None
            }
        }
    }
}

/// Even positions for renumbering a column: 1.0, 2.0, …, count.
pub fn evenly(count: usize) -> Vec<f64> {
    (1..=count).map(|index| index as f64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_column_gets_first_position() {
        assert_eq!(between(None, None), Some(1.0));
    }

    #[test]
    fn append_and_prepend_use_step() {
        assert_eq!(between(Some(3.0), None), Some(4.0));
        assert_eq!(between(None, Some(3.0)), Some(2.0));
    }

    #[test]
    fn insertion_between_takes_midpoint() {
        assert_eq!(between(Some(1.0), Some(2.0)), Some(1.5));
        assert_eq!(between(Some(2.0), Some(2.5)), Some(2.25));
    }

    #[test]
    fn exhausted_gap_requires_renumbering() {
        assert_eq!(between(Some(1.0), Some(1.0 + 1e-12)), None);
        assert_eq!(between(Some(2.0), Some(2.0)), None);
        assert_eq!(
            between(Some(3.0), Some(2.0)),
            None,
            "соседи должны быть по порядку"
        );
    }

    #[test]
    fn evenly_numbers_column_from_one() {
        assert_eq!(evenly(0), Vec::<f64>::new());
        assert_eq!(evenly(3), vec![1.0, 2.0, 3.0]);
    }
}
