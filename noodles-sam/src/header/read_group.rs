//! SAM header read group and fields.

mod platform;
mod tag;

use std::{collections::HashMap, convert::TryFrom, error, fmt};

pub use self::{platform::Platform, tag::Tag};

use super::{record, Record};

/// A SAM header read group.
///
/// A read group typically defines the set of reads that came from the same run on a sequencing
/// instrument. The read group ID is guaranteed to be set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadGroup {
    id: String,
    barcode: Option<String>,
    sequencing_center: Option<String>,
    description: Option<String>,
    produced_at: Option<String>,
    flow_order: Option<String>,
    key_sequence: Option<String>,
    library: Option<String>,
    program: Option<String>,
    predicted_median_insert_size: Option<String>,
    platform: Option<Platform>,
    platform_model: Option<String>,
    platform_unit: Option<String>,
    sample: Option<String>,
    fields: HashMap<Tag, String>,
}

impl ReadGroup {
    /// Creates a read group with an ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert_eq!(read_group.id(), "rg0");
    /// ```
    pub fn new(id: String) -> Self {
        Self {
            id,
            barcode: None,
            sequencing_center: None,
            description: None,
            produced_at: None,
            flow_order: None,
            key_sequence: None,
            library: None,
            program: None,
            predicted_median_insert_size: None,
            platform: None,
            platform_model: None,
            platform_unit: None,
            sample: None,
            fields: HashMap::new(),
        }
    }

    /// Returns the read group ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert_eq!(read_group.id(), "rg0");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a mutable reference to the read group ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    ///
    /// let mut read_group = ReadGroup::new(String::from("rg0"));
    /// assert_eq!(read_group.id(), "rg0");
    ///
    /// *read_group.id_mut() = String::from("rg1");
    /// assert_eq!(read_group.id(), "rg1");
    /// ```
    //
    pub fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }

    /// Returns the barcode sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.barcode().is_none());
    /// ```
    pub fn barcode(&self) -> Option<&str> {
        self.barcode.as_deref()
    }

    /// Returns the sequencing center.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.sequencing_center().is_none());
    /// ```
    pub fn sequencing_center(&self) -> Option<&str> {
        self.sequencing_center.as_deref()
    }

    /// Returns the description.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.description().is_none());
    /// ```
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the datatime of run.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.produced_at().is_none());
    /// ```
    pub fn produced_at(&self) -> Option<&str> {
        self.produced_at.as_deref()
    }

    /// Returns the flow order.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.flow_order().is_none());
    /// ```
    pub fn flow_order(&self) -> Option<&str> {
        self.flow_order.as_deref()
    }

    /// Returns the key sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.key_sequence().is_none());
    /// ```
    pub fn key_sequence(&self) -> Option<&str> {
        self.key_sequence.as_deref()
    }

    /// Returns the library.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.library().is_none());
    /// ```
    pub fn library(&self) -> Option<&str> {
        self.library.as_deref()
    }

    /// Returns the programs used.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.program().is_none());
    /// ```
    pub fn program(&self) -> Option<&str> {
        self.program.as_deref()
    }

    /// Returns the predicted median insert size.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.predicted_median_insert_size().is_none());
    /// ```
    pub fn predicted_median_insert_size(&self) -> Option<&str> {
        self.predicted_median_insert_size.as_deref()
    }

    /// Returns the platform used.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.platform().is_none());
    /// ```
    pub fn platform(&self) -> Option<Platform> {
        self.platform
    }

    /// Returns the platform model.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.platform_model().is_none());
    /// ```
    pub fn platform_model(&self) -> Option<&str> {
        self.platform_model.as_deref()
    }

    /// Returns the platform unit.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.platform_unit().is_none());
    /// ```
    pub fn platform_unit(&self) -> Option<&str> {
        self.platform_unit.as_deref()
    }

    /// Returns the sample.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::ReadGroup;
    /// let read_group = ReadGroup::new(String::from("rg0"));
    /// assert!(read_group.sample().is_none());
    /// ```
    pub fn sample(&self) -> Option<&str> {
        self.sample.as_deref()
    }

    /// Returns the raw fields of the read group.
    ///
    /// This includes any field that is not specially handled by the structure itself. For example,
    /// this will not include the ID field, as it is parsed and available as [`id`].
    ///
    /// [`id`]: #method.id
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::{read_group::Tag, ReadGroup};
    ///
    /// let mut read_group = ReadGroup::new(String::from("rg0"));
    /// read_group.insert(Tag::Other(String::from("zn")), String::from("noodles"));
    ///
    /// let fields = read_group.fields();
    /// assert_eq!(fields.len(), 1);
    /// assert_eq!(
    ///     read_group.get(&Tag::Other(String::from("zn"))),
    ///     Some(&String::from("noodles"))
    /// );
    ///
    /// assert_eq!(fields.get(&Tag::Id), None);
    /// assert_eq!(read_group.id(), "rg0");
    /// ```
    pub fn fields(&self) -> &HashMap<Tag, String> {
        &self.fields
    }

    /// Returns a reference to the raw field value mapped to the given key.
    ///
    /// This can only be used for fields with unparsed values. For example, [`id`] must be used
    /// instead of `get(Tag::Id)`.
    ///
    /// [`id`]: #method.id
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::{read_group::Tag, ReadGroup};
    ///
    /// let mut read_group = ReadGroup::new(String::from("rg0"));
    /// read_group.insert(Tag::Other(String::from("zn")), String::from("noodles"));
    ///
    /// assert_eq!(
    ///     read_group.get(&Tag::Other(String::from("zn"))),
    ///     Some(&String::from("noodles"))
    /// );
    /// assert_eq!(read_group.get(&Tag::Id), None);
    /// ```
    pub fn get(&self, tag: &Tag) -> Option<&String> {
        self.fields.get(tag)
    }

    /// Inserts a tag-raw value pair into the read group.
    ///
    /// This follows similar semantics to [`std::collections::HashMap::insert`].
    ///
    /// [`std::collections::HashMap::insert`]: https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html#method.insert
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::header::{read_group::Tag, ReadGroup};
    /// let mut read_group = ReadGroup::new(String::from("rg0"));
    /// read_group.insert(Tag::Other(String::from("zn")), String::from("noodles"));
    /// ```
    pub fn insert(&mut self, tag: Tag, value: String) -> Option<String> {
        self.fields.insert(tag, value)
    }
}

impl fmt::Display for ReadGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", record::Kind::ReadGroup)?;
        write!(f, "\t{}:{}", Tag::Id, self.id)?;

        if let Some(barcode) = self.barcode() {
            write!(f, "\t{}:{}", Tag::Barcode, barcode)?;
        }

        if let Some(sequencing_center) = self.sequencing_center() {
            write!(f, "\t{}:{}", Tag::SequencingCenter, sequencing_center)?;
        }

        if let Some(description) = self.description() {
            write!(f, "\t{}:{}", Tag::Description, description)?;
        }

        if let Some(produced_at) = self.produced_at() {
            write!(f, "\t{}:{}", Tag::ProducedAt, produced_at)?;
        }

        if let Some(flow_order) = self.flow_order() {
            write!(f, "\t{}:{}", Tag::FlowOrder, flow_order)?;
        }

        if let Some(key_sequence) = self.key_sequence() {
            write!(f, "\t{}:{}", Tag::KeySequence, key_sequence)?;
        }

        if let Some(library) = self.library() {
            write!(f, "\t{}:{}", Tag::Library, library)?;
        }

        if let Some(program) = self.program() {
            write!(f, "\t{}:{}", Tag::Program, program)?;
        }

        if let Some(predicted_median_insert_size) = self.predicted_median_insert_size() {
            write!(
                f,
                "\t{}:{}",
                Tag::PredictedMedianInsertSize,
                predicted_median_insert_size
            )?;
        }

        if let Some(platform) = self.platform() {
            write!(f, "\t{}:{}", Tag::Platform, platform)?;
        }

        if let Some(platform_model) = self.platform_model() {
            write!(f, "\t{}:{}", Tag::PlatformModel, platform_model)?;
        }

        if let Some(platform_unit) = self.platform_unit() {
            write!(f, "\t{}:{}", Tag::PlatformUnit, platform_unit)?;
        }

        if let Some(sample) = self.sample() {
            write!(f, "\t{}:{}", Tag::Sample, sample)?;
        }

        for (tag, value) in &self.fields {
            write!(f, "\t{}:{}", tag, value)?;
        }

        Ok(())
    }
}

/// An error returned when a raw SAM header read group fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TryFromRecordError {
    /// The record is invalid.
    InvalidRecord,
    /// A required tag is missing.
    MissingRequiredTag(Tag),
    /// A tag is invalid.
    InvalidTag(tag::ParseError),
    /// The platform is invalid.
    InvalidPlatform(platform::ParseError),
}

impl error::Error for TryFromRecordError {}

impl fmt::Display for TryFromRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRecord => f.write_str("invalid record"),
            Self::MissingRequiredTag(tag) => write!(f, "missing required tag: {:?}", tag),
            Self::InvalidTag(e) => write!(f, "{}", e),
            Self::InvalidPlatform(e) => write!(f, "invalid platform: {}", e),
        }
    }
}

impl TryFrom<Record> for ReadGroup {
    type Error = TryFromRecordError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        match record.into() {
            (record::Kind::ReadGroup, record::Value::Map(fields)) => parse_map(fields),
            _ => Err(TryFromRecordError::InvalidRecord),
        }
    }
}

fn parse_map(raw_fields: Vec<(String, String)>) -> Result<ReadGroup, TryFromRecordError> {
    let mut id = None;
    let mut barcode = None;
    let mut sequencing_center = None;
    let mut description = None;
    let mut produced_at = None;
    let mut flow_order = None;
    let mut key_sequence = None;
    let mut library = None;
    let mut program = None;
    let mut predicted_median_insert_size = None;
    let mut platform = None;
    let mut platform_model = None;
    let mut platform_unit = None;
    let mut sample = None;
    let mut fields = HashMap::new();

    for (raw_tag, value) in raw_fields {
        let tag = raw_tag.parse().map_err(TryFromRecordError::InvalidTag)?;

        match tag {
            Tag::Id => {
                id = Some(value);
            }
            Tag::Barcode => {
                barcode = Some(value);
            }
            Tag::SequencingCenter => {
                sequencing_center = Some(value);
            }
            Tag::Description => {
                description = Some(value);
            }
            Tag::ProducedAt => {
                produced_at = Some(value);
            }
            Tag::FlowOrder => {
                flow_order = Some(value);
            }
            Tag::KeySequence => {
                key_sequence = Some(value);
            }
            Tag::Library => {
                library = Some(value);
            }
            Tag::Program => {
                program = Some(value);
            }
            Tag::PredictedMedianInsertSize => {
                predicted_median_insert_size = Some(value);
            }
            Tag::Platform => {
                platform = value
                    .parse()
                    .map(Some)
                    .map_err(TryFromRecordError::InvalidPlatform)?;
            }
            Tag::PlatformModel => {
                platform_model = Some(value);
            }
            Tag::PlatformUnit => {
                platform_unit = Some(value);
            }
            Tag::Sample => {
                sample = Some(value);
            }
            _ => {
                fields.insert(tag, value);
            }
        }
    }

    Ok(ReadGroup {
        id: id.ok_or_else(|| TryFromRecordError::MissingRequiredTag(Tag::Id))?,
        barcode,
        sequencing_center,
        description,
        produced_at,
        flow_order,
        key_sequence,
        library,
        program,
        predicted_median_insert_size,
        platform,
        platform_model,
        platform_unit,
        sample,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let mut read_group = ReadGroup::new(String::from("rg0"));

        read_group
            .fields
            .insert(Tag::Program, String::from("noodles"));

        let actual = format!("{}", read_group);
        let expected = "@RG\tID:rg0\tPG:noodles";

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_try_from_record_for_read_group_with_invalid_record() {
        let record = Record::new(
            record::Kind::Comment,
            record::Value::String(String::from("noodles-sam")),
        );

        assert_eq!(
            ReadGroup::try_from(record),
            Err(TryFromRecordError::InvalidRecord)
        );
    }

    #[test]
    fn test_try_from_record_for_read_group_with_no_id() {
        let record = Record::new(
            record::Kind::ReadGroup,
            record::Value::Map(vec![(String::from("PG"), String::from("noodles"))]),
        );

        assert_eq!(
            ReadGroup::try_from(record),
            Err(TryFromRecordError::MissingRequiredTag(Tag::Id))
        );
    }
}
