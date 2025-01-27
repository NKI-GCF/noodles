//! Coordinate-sorted index (CSI) reference sequence and fields.

pub mod bin;
mod metadata;

pub use self::{bin::Bin, metadata::Metadata};

use std::{
    error, fmt,
    ops::{Bound, RangeBounds},
};

use bit_vec::BitVec;
use noodles_bgzf as bgzf;

use crate::BinningIndexReferenceSequence;

const MIN_POSITION: i64 = 1;

/// A CSI reference sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceSequence {
    bins: Vec<Bin>,
    metadata: Option<Metadata>,
}

/// An error returned when a query fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryError {
    /// The start position is invalid.
    InvalidStartPosition(i64, i64),
    /// The end position is invalid.
    InvalidEndPosition(i64, i64),
}

impl error::Error for QueryError {}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStartPosition(min_position, start) => {
                write!(
                    f,
                    "expected start position >= {}, got {}",
                    min_position, start
                )
            }
            Self::InvalidEndPosition(max_position, end) => {
                write!(f, "expected end position <= {}, got {}", max_position, end)
            }
        }
    }
}

impl ReferenceSequence {
    fn max_position(min_shift: i32, depth: i32) -> i64 {
        let min_shift = i64::from(min_shift);
        let depth = i64::from(depth);
        (1 << (min_shift + 3 * depth)) - 1
    }

    /// Creates a CSI reference sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::index::ReferenceSequence;
    /// let reference_sequence = ReferenceSequence::new(Vec::new(), None);
    /// ```
    pub fn new(bins: Vec<Bin>, metadata: Option<Metadata>) -> Self {
        Self { bins, metadata }
    }

    /// Returns the list of bins in the reference sequence.
    ///
    /// This list does not include the metadata pseudo-bin. Use [`Self::metadata`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::index::ReferenceSequence;
    /// let reference_sequence = ReferenceSequence::new(Vec::new(), None);
    /// assert!(reference_sequence.bins().is_empty());
    /// ```
    pub fn bins(&self) -> &[Bin] {
        &self.bins
    }

    /// Returns a list of bins in this reference sequence that intersects the given range.
    ///
    /// The interval values are 1-based.
    ///
    /// # Examples
    ///
    /// ```
    /// # use noodles_csi::index::reference_sequence;
    /// use noodles_csi::index::ReferenceSequence;
    /// let reference_sequence = ReferenceSequence::new(Vec::new(), None);
    /// let query_bins = reference_sequence.query(14, 5, 8..=13)?;
    /// assert!(query_bins.is_empty());
    /// # Ok::<(), reference_sequence::QueryError>(())
    /// ```
    pub fn query<B>(&self, min_shift: i32, depth: i32, interval: B) -> Result<Vec<&Bin>, QueryError>
    where
        B: RangeBounds<i64>,
    {
        let start = match interval.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => MIN_POSITION,
        };

        if start < MIN_POSITION {
            return Err(QueryError::InvalidStartPosition(MIN_POSITION, start));
        }

        let max_position = Self::max_position(min_shift, depth);

        let end = match interval.end_bound() {
            Bound::Included(e) => *e,
            Bound::Excluded(e) => *e - 1,
            Bound::Unbounded => max_position,
        };

        if end > max_position {
            return Err(QueryError::InvalidEndPosition(max_position, end));
        }

        let max_bin_id = Bin::max_id(depth);
        let mut region_bins = BitVec::from_elem(max_bin_id as usize, false);

        reg2bins(start - 1, end, min_shift, depth, &mut region_bins);

        let query_bins = self
            .bins()
            .iter()
            .filter(|b| region_bins[b.id() as usize])
            .collect();

        Ok(query_bins)
    }
}

impl BinningIndexReferenceSequence for ReferenceSequence {
    /// Returns the optional metadata for the reference sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// use noodles_csi::{
    ///     index::{reference_sequence::Metadata, ReferenceSequence},
    ///     BinningIndexReferenceSequence,
    /// };
    ///
    /// let reference_sequence = ReferenceSequence::new(Vec::new(), Some(Metadata::new(
    ///     bgzf::VirtualPosition::from(610),
    ///     bgzf::VirtualPosition::from(1597),
    ///     55,
    ///     0,
    /// )));
    ///
    /// assert!(reference_sequence.metadata().is_some());
    /// ```
    fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Returns the start position of the first record in the last linear bin.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// use noodles_csi::{
    ///     index::{reference_sequence::Bin, ReferenceSequence},
    ///     BinningIndexReferenceSequence,
    /// };
    ///
    /// let reference_sequence = ReferenceSequence::new(Vec::new(), None);
    /// assert!(reference_sequence.first_record_in_last_linear_bin_start_position().is_none());
    ///
    /// let bins = vec![
    ///     Bin::new(0, bgzf::VirtualPosition::from(8), Vec::new()),
    ///     Bin::new(1, bgzf::VirtualPosition::from(13), Vec::new()),
    /// ];
    /// let reference_sequence = ReferenceSequence::new(bins, None);
    /// assert_eq!(
    ///     reference_sequence.first_record_in_last_linear_bin_start_position(),
    ///     Some(bgzf::VirtualPosition::from(13))
    /// );
    /// ```
    fn first_record_in_last_linear_bin_start_position(&self) -> Option<bgzf::VirtualPosition> {
        self.bins().last().map(|bin| bin.loffset())
    }
}

// `CSIv1.pdf` (2020-07-21)
// [beg, end), 0-based
#[allow(clippy::many_single_char_names)]
fn reg2bins(beg: i64, mut end: i64, min_shift: i32, depth: i32, bins: &mut BitVec) {
    end -= 1;

    let mut l = 0;
    let mut t = 0;
    let mut s = min_shift + depth * 3;

    while l <= depth {
        let b = t + (beg >> s);
        let e = t + (end >> s);

        for i in b..=e {
            bins.set(i as usize, true);
        }

        s -= 3;
        t += 1 << (l * 3);
        l += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN_SHIFT: i32 = 14;
    const DEPTH: i32 = 5;

    #[test]
    fn test_max_position() {
        let max_position = ReferenceSequence::max_position(MIN_SHIFT, DEPTH);
        assert_eq!(max_position, 536870911);
    }

    #[test]
    fn test_query() {
        let reference_sequence = ReferenceSequence::new(Vec::new(), None);

        assert_eq!(
            reference_sequence.query(MIN_SHIFT, DEPTH, 0..=8),
            Err(QueryError::InvalidStartPosition(1, 0))
        );

        let end = i64::from(i32::MAX);
        assert_eq!(
            reference_sequence.query(MIN_SHIFT, DEPTH, 1..=end),
            Err(QueryError::InvalidEndPosition(536870911, end))
        );
    }

    #[test]
    fn test_reg2bins() {
        // +------------------------------------------------------------------------------------...
        // | 0                                                                                  ...
        // | 0-1023                                                                             ...
        // +-------------------------------------------------------------------------+----------...
        // | 1                                                                       | 2        ...
        // | 0-127                                                                   | 128-255  ...
        // +--------+--------+--------+--------+--------+--------+---------+---------+---------+...
        // | 9      | 10     | 11     | 12     | 13     | 14     | 15      | 16      | 17      |...
        // | 0-15   | 16-31  | 32-47  | 48-63  | 64-79  | 80-95  | 96-111  | 112-127 | 128-143 |...
        // +--------+--------+--------+--------+--------+--------+---------+---------+---------+...

        const MIN_SHIFT: i32 = 4;
        const DEPTH: i32 = 2;

        fn t(start: i64, end: i64, expected_bin_ids: &[u32]) {
            let max_bin_id = Bin::max_id(DEPTH);

            let mut actual = BitVec::from_elem(max_bin_id as usize, false);
            reg2bins(start, end, MIN_SHIFT, DEPTH, &mut actual);

            let mut expected = BitVec::from_elem(max_bin_id as usize, false);

            for &bin_id in expected_bin_ids {
                expected.set(bin_id as usize, true);
            }

            assert_eq!(actual, expected);
        }

        t(0, 16, &[0, 1, 9]);
        t(8, 13, &[0, 1, 9]);
        t(35, 67, &[0, 1, 11, 12, 13]);
        t(48, 143, &[0, 1, 2, 12, 13, 14, 15, 16, 17]);
    }
}
