//! GFF record attribute entry.

use std::str::FromStr;

const SEPARATOR: char = '=';

/// A GFF record attribute entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    key: String,
    value: String,
}

impl Entry {
    /// Creates a GFF record attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gff::record::attributes::Entry;
    /// let entry = Entry::new(String::from("gene_name"), String::from("gene0"));
    /// ```
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    /// Returns the key of the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gff::record::attributes::Entry;
    /// let entry = Entry::new(String::from("gene_name"), String::from("gene0"));
    /// assert_eq!(entry.key(), "gene_name");
    /// ```
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the value of the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gff::record::attributes::Entry;
    /// let entry = Entry::new(String::from("gene_name"), String::from("gene0"));
    /// assert_eq!(entry.value(), "gene0");
    /// ```
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// An error returned when a raw GFF record attribute entry fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The entry key is missing.
    MissingKey,
    /// The entry value is missing.
    MissingValue,
}

impl FromStr for Entry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let mut components = s.splitn(2, SEPARATOR);

        let key = components
            .next()
            .and_then(|s| if s.is_empty() { None } else { Some(s.into()) })
            .ok_or_else(|| ParseError::MissingKey)?;

        let value = components
            .next()
            .map(|s| s.into())
            .ok_or_else(|| ParseError::MissingValue)?;

        Ok(Entry::new(key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() -> Result<(), ParseError> {
        assert_eq!(
            "gene_name=gene0".parse::<Entry>()?,
            Entry::new(String::from("gene_name"), String::from("gene0"))
        );

        assert_eq!("".parse::<Entry>(), Err(ParseError::Empty));
        assert_eq!("=gene0".parse::<Entry>(), Err(ParseError::MissingKey));
        assert_eq!("gene_name".parse::<Entry>(), Err(ParseError::MissingValue));

        Ok(())
    }
}
