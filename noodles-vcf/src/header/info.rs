mod key;
mod ty;

pub use self::ty::Type;

use std::{collections::HashMap, convert::TryFrom, error, fmt};

use crate::record::info;

use super::{number, record, Number, Record};

use self::key::Key;

/// A VCF header information record (`INFO`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Info {
    id: info::field::Key,
    number: Number,
    ty: Type,
    description: String,
    fields: HashMap<String, String>,
}

impl Info {
    /// Creates a VCF header information record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     header::{info::Type, Info, Number},
    ///     record::info::field::Key,
    /// };
    ///
    /// let info = Info::new(
    ///     Key::SamplesWithDataCount,
    ///     Number::Count(1),
    ///     Type::Integer,
    ///     String::from("Number of samples with data"),
    /// );
    /// ```
    pub fn new(id: info::field::Key, number: Number, ty: Type, description: String) -> Self {
        Self {
            id,
            number,
            ty,
            description,
            fields: HashMap::new(),
        }
    }

    /// Returns the information field key.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     header::{info::Type, Info, Number},
    ///     record::info::field::Key,
    /// };
    ///
    /// let info = Info::new(
    ///     Key::SamplesWithDataCount,
    ///     Number::Count(1),
    ///     Type::Integer,
    ///     String::from("Number of samples with data"),
    /// );
    ///
    /// assert_eq!(info.id(), &Key::SamplesWithDataCount);
    /// ```
    pub fn id(&self) -> &info::field::Key {
        &self.id
    }

    /// Returns the cardinality of the information field value.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     header::{info::Type, Info, Number},
    ///     record::info::field::Key,
    /// };
    ///
    /// let info = Info::new(
    ///     Key::SamplesWithDataCount,
    ///     Number::Count(1),
    ///     Type::Integer,
    ///     String::from("Number of samples with data"),
    /// );
    ///
    /// assert_eq!(info.number(), Number::Count(1));
    /// ```
    pub fn number(&self) -> Number {
        self.number
    }

    /// Returns the type of the information field value.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     header::{info::Type, Info, Number},
    ///     record::info::field::Key,
    /// };
    ///
    /// let info = Info::new(
    ///     Key::SamplesWithDataCount,
    ///     Number::Count(1),
    ///     Type::Integer,
    ///     String::from("Number of samples with data"),
    /// );
    ///
    /// assert_eq!(info.ty(), Type::Integer);
    /// ```
    pub fn ty(&self) -> Type {
        self.ty
    }

    /// Returns the description of the information field.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     header::{info::Type, Info, Number},
    ///     record::info::field::Key,
    /// };
    ///
    /// let info = Info::new(
    ///     Key::SamplesWithDataCount,
    ///     Number::Count(1),
    ///     Type::Integer,
    ///     String::from("Number of samples with data"),
    /// );
    ///
    /// assert_eq!(info.description(), "Number of samples with data");
    /// ```
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the extra fields in the record.
    ///
    /// This includes fields other than `ID`, `Number`, `Type`, and `Description`.
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("##")?;
        f.write_str(record::Key::Info.as_ref())?;
        f.write_str("=<")?;

        write!(f, "{}={}", Key::Id, self.id)?;
        write!(f, ",{}={}", Key::Number, self.number)?;
        write!(f, ",{}={}", Key::Type, self.ty)?;
        write!(f, r#",{}="{}""#, Key::Description, self.description)?;

        for (key, value) in &self.fields {
            write!(f, r#",{}="{}""#, key, value)?;
        }

        f.write_str(">")?;

        Ok(())
    }
}

/// An error returned when a raw VCF header information record fails to parse.
#[derive(Debug)]
pub enum TryFromRecordError {
    /// The record key is invalid.
    InvalidRecordKey,
    /// The record value is invalid.
    InvalidRecordValue,
    /// A required field is missing.
    MissingField(Key),
    /// The ID is invalid.
    InvalidId(info::field::key::ParseError),
    /// The number is invalid.
    InvalidNumber(number::ParseError),
    /// The type is invalid.
    InvalidType(ty::ParseError),
}

impl error::Error for TryFromRecordError {}

impl fmt::Display for TryFromRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRecordKey => f.write_str("invalid record key"),
            Self::InvalidRecordValue => f.write_str("invalid record value"),
            Self::MissingField(key) => write!(f, "missing {} field", key),
            Self::InvalidId(e) => write!(f, "invalid ID: {}", e),
            Self::InvalidNumber(e) => write!(f, "invalid number: {}", e),
            Self::InvalidType(e) => write!(f, "invalid type: {}", e),
        }
    }
}

impl TryFrom<Record> for Info {
    type Error = TryFromRecordError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        let (key, value) = record.into();

        match key {
            record::Key::Info => match value {
                record::Value::Struct(fields) => parse_struct(fields),
                _ => Err(TryFromRecordError::InvalidRecordValue),
            },
            _ => Err(TryFromRecordError::InvalidRecordKey),
        }
    }
}

fn parse_struct(fields: Vec<(String, String)>) -> Result<Info, TryFromRecordError> {
    let mut it = fields.into_iter();

    let id = it
        .next()
        .ok_or_else(|| TryFromRecordError::MissingField(Key::Id))
        .and_then(|(k, v)| match k.parse() {
            Ok(Key::Id) => v.parse().map_err(TryFromRecordError::InvalidId),
            _ => Err(TryFromRecordError::MissingField(Key::Id)),
        })?;

    let number = it
        .next()
        .ok_or_else(|| TryFromRecordError::MissingField(Key::Number))
        .and_then(|(k, v)| match k.parse() {
            Ok(Key::Number) => v.parse().map_err(TryFromRecordError::InvalidNumber),
            _ => Err(TryFromRecordError::MissingField(Key::Id)),
        })?;

    let ty = it
        .next()
        .ok_or_else(|| TryFromRecordError::MissingField(Key::Type))
        .and_then(|(k, v)| match k.parse() {
            Ok(Key::Type) => v.parse().map_err(TryFromRecordError::InvalidType),
            _ => Err(TryFromRecordError::MissingField(Key::Type)),
        })?;

    let description = it
        .next()
        .ok_or_else(|| TryFromRecordError::MissingField(Key::Description))
        .and_then(|(k, v)| match k.parse() {
            Ok(Key::Description) => Ok(v),
            _ => Err(TryFromRecordError::MissingField(Key::Description)),
        })?;

    Ok(Info {
        id,
        number,
        ty,
        description,
        fields: it.collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_record() -> Record {
        Record::new(
            record::Key::Info,
            record::Value::Struct(vec![
                (String::from("ID"), String::from("NS")),
                (String::from("Number"), String::from("1")),
                (String::from("Type"), String::from("Integer")),
                (
                    String::from("Description"),
                    String::from("Number of samples with data"),
                ),
            ]),
        )
    }

    #[test]
    fn test_fmt() -> Result<(), TryFromRecordError> {
        let record = build_record();
        let info = Info::try_from(record)?;

        let expected =
            r#"##INFO=<ID=NS,Number=1,Type=Integer,Description="Number of samples with data">"#;

        assert_eq!(info.to_string(), expected);

        Ok(())
    }

    #[test]
    fn test_try_from_record_for_info() -> Result<(), TryFromRecordError> {
        let record = build_record();
        let info = Info::try_from(record)?;

        assert_eq!(info.id(), &info::field::Key::SamplesWithDataCount);
        assert_eq!(info.number(), Number::Count(1));
        assert_eq!(info.ty(), Type::Integer);
        assert_eq!(info.description(), "Number of samples with data");

        Ok(())
    }

    #[test]
    fn test_try_from_record_for_info_with_extra_fields() -> Result<(), TryFromRecordError> {
        let record = Record::new(
            record::Key::Info,
            record::Value::Struct(vec![
                (String::from("ID"), String::from("NS")),
                (String::from("Number"), String::from("1")),
                (String::from("Type"), String::from("Integer")),
                (
                    String::from("Description"),
                    String::from("Number of samples with data"),
                ),
                (String::from("Source"), String::from("dbsnp")),
                (String::from("Version"), String::from("138")),
            ]),
        );

        let info = Info::try_from(record)?;

        let fields = info.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields["Source"], "dbsnp");
        assert_eq!(fields["Version"], "138");

        Ok(())
    }
}
