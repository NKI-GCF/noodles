mod key;

use std::{collections::HashMap, convert::TryFrom, error, fmt, num};

use super::{record, Record};

use self::key::Key;

/// A VCF header contig record (`contig`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contig {
    id: String,
    len: Option<i32>,
    fields: HashMap<String, String>,
}

#[allow(clippy::len_without_is_empty)]
impl Contig {
    /// Creates a VCF header contig record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::Contig;
    /// let contig = Contig::new(String::from("sq0"));
    /// ```
    pub fn new(id: String) -> Self {
        Self {
            id,
            len: None,
            fields: HashMap::new(),
        }
    }

    /// Returns the ID of the contig.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::Contig;
    /// let contig = Contig::new(String::from("sq0"));
    /// assert_eq!(contig.id(), "sq0");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the length of the contig, if it is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::Contig;
    /// let contig = Contig::new(String::from("sq0"));
    /// assert_eq!(contig.len(), None);
    /// ```
    pub fn len(&self) -> Option<i32> {
        self.len
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|s| &**s)
    }
}

impl fmt::Display for Contig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("##")?;
        f.write_str(record::Key::Contig.as_ref())?;
        f.write_str("=<")?;

        write!(f, "{}={}", Key::Id, self.id)?;

        if let Some(len) = self.len {
            write!(f, ",{}={}", Key::Length, len)?;
        }

        for (key, value) in &self.fields {
            write!(f, r#",{}="{}""#, key, value)?;
        }

        f.write_str(">")?;

        Ok(())
    }
}

/// An error returned when a raw VCF header contig record fails to parse.
#[derive(Debug)]
pub enum TryFromRecordError {
    /// The record key is invalid.
    InvalidRecordKey,
    /// The record value is invalid.
    InvalidRecordValue,
    /// A key is invalid.
    InvalidKey(key::ParseError),
    /// The length is invalid.
    InvalidLength(num::ParseIntError),
    /// A required field is missing.
    MissingField(Key),
}

impl error::Error for TryFromRecordError {}

impl fmt::Display for TryFromRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid contig header: ")?;

        match self {
            Self::InvalidRecordKey => f.write_str("invalid record key"),
            Self::InvalidRecordValue => f.write_str("invalid record value"),
            Self::MissingField(key) => write!(f, "missing {} field", key),
            Self::InvalidKey(e) => write!(f, "invalid key: {}", e),
            Self::InvalidLength(e) => write!(f, "invalid length: {}", e),
        }
    }
}

impl TryFrom<Record> for Contig {
    type Error = TryFromRecordError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        let (key, value) = record.into();

        match key {
            record::Key::Contig => match value {
                record::Value::Struct(fields) => parse_struct(fields),
                _ => Err(TryFromRecordError::InvalidRecordValue),
            },
            _ => Err(TryFromRecordError::InvalidRecordKey),
        }
    }
}

fn parse_struct(fields: Vec<(String, String)>) -> Result<Contig, TryFromRecordError> {
    let mut contig = Contig {
        id: String::from("unknown"),
        len: None,
        fields: HashMap::new(),
    };

    let mut has_id = false;

    for (raw_key, value) in fields {
        let key = raw_key.parse().map_err(TryFromRecordError::InvalidKey)?;

        match key {
            Key::Id => {
                contig.id = value;
                has_id = true;
            }
            Key::Length => {
                contig.len = value
                    .parse()
                    .map(Some)
                    .map_err(TryFromRecordError::InvalidLength)?;
            }
            Key::Other(k) => {
                contig.fields.insert(k, value);
            }
        }
    }

    if !has_id {
        return Err(TryFromRecordError::MissingField(Key::Id));
    }

    Ok(contig)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_record() -> Record {
        Record::new(
            record::Key::Contig,
            record::Value::Struct(vec![
                (String::from("ID"), String::from("sq0")),
                (String::from("length"), String::from("13")),
                (
                    String::from("md5"),
                    String::from("d7eba311421bbc9d3ada44709dd61534"),
                ),
            ]),
        )
    }

    #[test]
    fn test_fmt() -> Result<(), TryFromRecordError> {
        let record = build_record();
        let contig = Contig::try_from(record)?;

        let expected = r#"##contig=<ID=sq0,length=13,md5="d7eba311421bbc9d3ada44709dd61534">"#;

        assert_eq!(contig.to_string(), expected);

        Ok(())
    }

    #[test]
    fn test_try_from_record_for_contig() -> Result<(), TryFromRecordError> {
        let record = build_record();
        let contig = Contig::try_from(record)?;

        assert_eq!(contig.id(), "sq0");
        assert_eq!(contig.len(), Some(13));
        assert_eq!(contig.get("md5"), Some("d7eba311421bbc9d3ada44709dd61534"));

        Ok(())
    }
}
