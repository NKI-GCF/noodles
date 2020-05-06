use std::{
    ffi::CString,
    io::{self, Write},
};

use byteorder::{LittleEndian, WriteBytesExt};
use noodles_bgzf as bgzf;
use noodles_sam::{
    self as sam,
    header::ReferenceSequences,
    record::{MateReferenceSequenceName, QualityScores, Sequence},
};

use super::MAGIC_NUMBER;

pub struct Writer<W>
where
    W: Write,
{
    inner: bgzf::Writer<W>,
}

impl<W> Writer<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            inner: bgzf::Writer::new(writer),
        }
    }

    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }

    pub fn try_finish(&mut self) -> io::Result<()> {
        self.inner.try_finish()
    }

    pub fn write_header(&mut self, header: &sam::Header) -> io::Result<()> {
        self.inner.write_all(MAGIC_NUMBER)?;

        let text = header.to_string();
        let l_text = text.len() as i32;
        self.inner.write_i32::<LittleEndian>(l_text)?;

        self.inner.write_all(text.as_bytes())?;

        write_references(&mut self.inner, header.reference_sequences())
    }

    pub fn write_record(
        &mut self,
        reference_sequences: &ReferenceSequences,
        record: &sam::Record,
    ) -> io::Result<()> {
        let c_name = match record.name().as_ref() {
            Some(name) => CString::new(name.as_str())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            None => CString::default(),
        };

        let reference_sequence_id = match record.reference_sequence_name().as_ref() {
            Some(name) => reference_sequences
                .get_full(name)
                .map(|(i, _, _)| i as i32)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid reference sequence id")
                })?,
            None => -1,
        };

        let mate_reference_sequence_id = match record.mate_reference_sequence_name() {
            MateReferenceSequenceName::Some(name) => reference_sequences
                .get_full(name)
                .map(|(i, _, _)| i as i32)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid reference sequence id")
                })?,
            MateReferenceSequenceName::Eq => reference_sequence_id,
            MateReferenceSequenceName::None => -1,
        };

        let read_name = c_name.as_bytes_with_nul();
        let l_read_name = read_name.len() as u8;
        let n_cigar_op = record.cigar().ops().len() as u16;
        let l_seq = record.sequence().len() as i32;

        let block_size = 4
            + 4
            + 1
            + 1
            + 2
            + 2
            + 2
            + 4
            + 4
            + 4
            + 4
            + (l_read_name as i32)
            + (4 * (n_cigar_op as i32))
            + ((l_seq + 1) / 2)
            + l_seq;
        self.inner.write_i32::<LittleEndian>(block_size)?;

        let ref_id = reference_sequence_id as i32;
        self.inner.write_i32::<LittleEndian>(ref_id)?;

        let pos = i32::from(record.position()) - 1;
        self.inner.write_i32::<LittleEndian>(pos)?;

        self.inner.write_u8(l_read_name)?;

        let mapq = u8::from(record.mapping_quality());
        self.inner.write_u8(mapq)?;

        let bin = record
            .position()
            .map(|start| {
                let end = record.cigar().mapped_len() as i32;
                region_to_bin(start, end) as u16
            })
            .unwrap_or(4680);

        self.inner.write_u16::<LittleEndian>(bin)?;

        self.inner.write_u16::<LittleEndian>(n_cigar_op)?;

        let flag = u16::from(record.flags());
        self.inner.write_u16::<LittleEndian>(flag)?;

        self.inner.write_i32::<LittleEndian>(l_seq)?;

        let next_ref_id = mate_reference_sequence_id as i32;
        self.inner.write_i32::<LittleEndian>(next_ref_id)?;

        let next_pos = i32::from(record.mate_position()) - 1;
        self.inner.write_i32::<LittleEndian>(next_pos)?;

        let tlen = record.template_len();
        self.inner.write_i32::<LittleEndian>(tlen)?;

        self.inner.write_all(read_name)?;

        write_cigar(&mut self.inner, record.cigar())?;
        write_seq(&mut self.inner, record.sequence())?;
        write_qual(&mut self.inner, record.quality_scores())?;

        Ok(())
    }
}

impl<W> Drop for Writer<W>
where
    W: Write,
{
    fn drop(&mut self) {
        let _ = self.try_finish();
    }
}

fn write_references<W>(writer: &mut W, reference_sequences: &ReferenceSequences) -> io::Result<()>
where
    W: Write,
{
    let n_ref = reference_sequences.len() as i32;
    writer.write_i32::<LittleEndian>(n_ref)?;

    for reference_sequence in reference_sequences.values() {
        let c_name = CString::new(reference_sequence.name())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let name = c_name.as_bytes_with_nul();

        let l_name = name.len() as i32;
        writer.write_i32::<LittleEndian>(l_name)?;
        writer.write_all(name)?;

        let l_ref = reference_sequence.len() as i32;
        writer.write_i32::<LittleEndian>(l_ref)?;
    }

    Ok(())
}

fn write_cigar<W>(writer: &mut W, cigar: &sam::Cigar) -> io::Result<()>
where
    W: Write,
{
    for op in cigar.ops() {
        let len = op.len() as u32;
        let kind = op.kind() as u32;
        let value = len << 4 | kind;
        writer.write_u32::<LittleEndian>(value)?;
    }

    Ok(())
}

fn write_seq<W>(writer: &mut W, sequence: &Sequence) -> io::Result<()>
where
    W: Write,
{
    use sam::record::sequence::Base as SamBase;

    fn base_to_u8(base: SamBase) -> u8 {
        match base {
            SamBase::Eq => 0,
            SamBase::A => 1,
            SamBase::C => 2,
            SamBase::M => 3,
            SamBase::G => 4,
            SamBase::R => 5,
            SamBase::S => 6,
            SamBase::V => 7,
            SamBase::T => 8,
            SamBase::W => 9,
            SamBase::Y => 10,
            SamBase::H => 11,
            SamBase::K => 12,
            SamBase::D => 13,
            SamBase::B => 14,
            _ => 15,
        }
    }

    for chunk in sequence.bases().chunks(2) {
        let l = base_to_u8(chunk[0]);

        let r = if let Some(c) = chunk.get(1) {
            base_to_u8(*c)
        } else {
            0
        };

        let value = l << 4 | r;

        writer.write_u8(value)?;
    }

    Ok(())
}

fn write_qual<W>(writer: &mut W, quality_scores: &QualityScores) -> io::Result<()>
where
    W: Write,
{
    for score in quality_scores.scores() {
        let value = u8::from(*score);
        writer.write_u8(value)?;
    }

    Ok(())
}

// See § 5.3 in SAMv1.pdf (accessed 2020-04-24).
#[allow(clippy::eq_op)]
fn region_to_bin(start: i32, mut end: i32) -> i32 {
    end -= 1;

    if start >> 14 == end >> 14 {
        ((1 << 15) - 1) / 7 + (start >> 14)
    } else if start >> 17 == end >> 17 {
        ((1 << 12) - 1) / 7 + (start >> 17)
    } else if start >> 20 == end >> 20 {
        ((1 << 9) - 1) / 7 + (start >> 20)
    } else if start >> 23 == end >> 23 {
        ((1 << 6) - 1) / 7 + (start >> 23)
    } else if start >> 26 == end >> 26 {
        ((1 << 3) - 1) / 7 + (start >> 26)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::{Reader, Record};

    use super::*;

    #[test]
    fn test_write_header() -> io::Result<()> {
        let mut writer = Writer::new(Vec::new());

        let header = sam::Header::default();
        writer.write_header(&header)?;
        writer.try_finish()?;

        let mut reader = Reader::new(writer.get_ref().as_slice());
        let actual = reader.read_header()?;

        let expected = "@HD\tVN:1.6\n";

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_write_record() -> io::Result<()> {
        let mut writer = Writer::new(Vec::new());

        let header = sam::Header::default();
        let sam_record = sam::Record::default();
        writer.write_record(header.reference_sequences(), &sam_record)?;
        writer.try_finish()?;

        let mut reader = Reader::new(writer.get_ref().as_slice());

        let mut record = Record::default();
        reader.read_record(&mut record)?;

        assert_eq!(record.name(), b"\0");
        assert_eq!(record.flags(), sam::Flags::default());
        assert_eq!(record.reference_sequence_id(), -1);
        assert_eq!(record.position(), -1);
        assert_eq!(record.mapping_quality(), 255);
        assert!(record.cigar().is_empty());
        assert_eq!(record.mate_reference_sequence_id(), -1);
        assert_eq!(record.mate_position(), -1);
        assert_eq!(record.template_len(), 0);
        assert!(record.sequence().is_empty());
        assert!(record.quality_scores().is_empty());
        assert!(record.data().is_empty());

        Ok(())
    }
}