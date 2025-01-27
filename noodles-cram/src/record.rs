//! CRAM record and fields.

mod builder;
mod convert;
pub mod feature;
mod flags;
mod next_mate_flags;
mod read_group_id;
pub mod resolve;
pub mod tag;

pub use self::{
    builder::Builder, feature::Feature, flags::Flags, next_mate_flags::NextMateFlags,
    read_group_id::ReadGroupId, tag::Tag,
};

use std::{fmt, str};

use noodles_bam as bam;
use noodles_sam as sam;

/// A CRAM record.
#[derive(Clone, PartialEq)]
pub struct Record {
    pub(crate) id: i64,
    pub(crate) bam_bit_flags: sam::record::Flags,
    pub(crate) cram_bit_flags: Flags,
    pub(crate) reference_sequence_id: Option<bam::record::ReferenceSequenceId>,
    pub(crate) read_length: usize,
    pub(crate) alignment_start: Option<sam::record::Position>,
    pub(crate) read_group: ReadGroupId,
    pub(crate) read_name: Vec<u8>,
    pub(crate) next_mate_bit_flags: NextMateFlags,
    pub(crate) next_fragment_reference_sequence_id: Option<bam::record::ReferenceSequenceId>,
    pub(crate) next_mate_alignment_start: Option<sam::record::Position>,
    pub(crate) template_size: i32,
    pub(crate) distance_to_next_fragment: i32,
    pub(crate) tags: Vec<Tag>,
    pub(crate) bases: Vec<u8>,
    pub(crate) features: Vec<Feature>,
    pub(crate) mapping_quality: sam::record::MappingQuality,
    pub(crate) quality_scores: Vec<u8>,
}

impl Record {
    /// Returns a builder to create a record from each of its fields.
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    /// Returns the BAM flags.
    ///
    /// This is also called the BAM bit flags.
    pub fn bam_flags(&self) -> sam::record::Flags {
        self.bam_bit_flags
    }

    /// Returns the CRAM flags.
    ///
    /// This is also called the CRAM bit flags or compression bit flags.
    pub fn flags(&self) -> Flags {
        self.cram_bit_flags
    }

    /// Returns the reference sequence ID.
    ///
    /// This is also called the reference ID. It is the position of the reference sequence in the
    /// SAM header.
    pub fn reference_sequence_id(&self) -> Option<bam::record::ReferenceSequenceId> {
        self.reference_sequence_id
    }

    /// Returns the read length.
    pub fn read_length(&self) -> usize {
        self.read_length
    }

    /// Returns the alignment start position.
    ///
    /// This value is 1-based.
    pub fn alignment_start(&self) -> Option<sam::record::Position> {
        self.alignment_start
    }

    /// Returns the alignment end position.
    ///
    /// This value is 1-based.
    pub fn alignment_end(&self) -> i32 {
        calculate_alignment_end(
            self.alignment_start().map(i32::from).unwrap_or_default(),
            self.read_length() as i32,
            self.features(),
        )
    }

    /// Returns the read group ID.
    ///
    /// This is also simply called the read group. It is the position of the read group in the SAM
    /// header.
    pub fn read_group_id(&self) -> ReadGroupId {
        self.read_group
    }

    /// Returns the read name.
    ///
    /// This may be the original read name or a generated one.
    pub fn read_name(&self) -> &[u8] {
        &self.read_name
    }

    /// Returns the next mate flags.
    ///
    /// This is also call the next mate bit flags.
    pub fn next_mate_flags(&self) -> NextMateFlags {
        self.next_mate_bit_flags
    }

    /// Returns the reference sequence ID of the next fragment.
    ///
    /// It is the position of the reference sequence in the SAM header.
    pub fn next_fragment_reference_sequence_id(&self) -> Option<bam::record::ReferenceSequenceId> {
        self.next_fragment_reference_sequence_id
    }

    /// Returns the alignment start position of the next mate.
    ///
    /// This value is 1-based.
    pub fn next_mate_alignment_start(&self) -> Option<sam::record::Position> {
        self.next_mate_alignment_start
    }

    /// Returns the template size.
    pub fn template_size(&self) -> i32 {
        self.template_size
    }

    /// Returns the distance to the next fragment.
    ///
    /// This is the number of records to the next fragment within a slice.
    pub fn distance_to_next_fragment(&self) -> i32 {
        self.distance_to_next_fragment
    }

    /// Returns the tag dictionary.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Returns the read bases.
    pub fn bases(&self) -> &[u8] {
        &self.bases
    }

    /// Returns the read features.
    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    pub(crate) fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
    }

    /// Returns the mapping quality.
    pub fn mapping_quality(&self) -> sam::record::MappingQuality {
        self.mapping_quality
    }

    /// Returns the per-base quality scores.
    pub fn quality_scores(&self) -> &[u8] {
        &self.quality_scores
    }
}

impl Default for Record {
    fn default() -> Self {
        Builder::default().build()
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read_name = str::from_utf8(self.read_name());

        fmt.debug_struct("Record")
            .field("id", &self.id)
            .field("bam_bit_flags", &self.bam_flags())
            .field("cram_bit_flags", &self.flags())
            .field("reference_id", &self.reference_sequence_id)
            .field("read_length", &self.read_length)
            .field("alignment_start", &self.alignment_start)
            .field("read_group", &self.read_group)
            .field("read_name", &read_name)
            .field("next_mate_bit_flags", &self.next_mate_flags())
            .field(
                "next_fragment_reference_sequence_id",
                &self.next_fragment_reference_sequence_id,
            )
            .field("next_mate_alignment_start", &self.next_mate_alignment_start)
            .field("template_size", &self.template_size)
            .field("distance_to_next_fragment", &self.distance_to_next_fragment)
            .field("tags", &self.tags)
            .field("bases", &self.bases)
            .field("features", &self.features)
            .field("mapping_quality", &self.mapping_quality)
            .field("quality_scores", &self.quality_scores)
            .finish()
    }
}

fn calculate_alignment_span(read_length: i32, features: &[Feature]) -> i32 {
    features
        .iter()
        .fold(read_length, |alignment_span, feature| match feature {
            Feature::Insertion(_, bases) => alignment_span - bases.len() as i32,
            Feature::InsertBase(_, _) => alignment_span - 1,
            Feature::Deletion(_, len) => alignment_span + len,
            Feature::ReferenceSkip(_, len) => alignment_span + len,
            Feature::SoftClip(_, bases) => alignment_span - bases.len() as i32,
            _ => alignment_span,
        })
}

fn calculate_alignment_end(alignment_start: i32, read_length: i32, features: &[Feature]) -> i32 {
    let alignment_span = calculate_alignment_span(read_length, features);
    alignment_start + alignment_span - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_alignment_span() {
        let features = [];
        assert_eq!(calculate_alignment_span(4, &features), 4);

        let features = [Feature::HardClip(1, 4)];
        assert_eq!(calculate_alignment_span(4, &features), 4);

        let features = [
            Feature::Insertion(1, vec![b'A', b'C']),
            Feature::InsertBase(4, b'G'),
            Feature::Deletion(6, 3),
            Feature::ReferenceSkip(10, 5),
            Feature::SoftClip(16, vec![b'A', b'C', b'G', b'T']),
        ];
        assert_eq!(calculate_alignment_span(20, &features), 21);
    }

    #[test]
    fn test_calculate_alignment_end() {
        let features = [];
        assert_eq!(calculate_alignment_end(1, 4, &features), 4);
    }
}
