mod key;
mod value;

use std::{error, fmt, str::FromStr};

use crate::header::info::Type;

use self::{key::Key, value::Value};

const SEPARATOR: char = '=';
const MAX_COMPONENTS: usize = 2;

#[derive(Debug)]
pub struct Field {
    key: Key,
    value: Value,
}

impl Field {
    pub fn new(key: Key, value: Value) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug)]
pub enum ParseError {
    MissingKey,
    InvalidKey(key::ParseError),
    MissingValue,
    InvalidValue(value::ParseError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid info: ")?;

        match self {
            ParseError::MissingKey => f.write_str("missing key"),
            ParseError::InvalidKey(e) => write!(f, "invalid key: {}", e),
            ParseError::MissingValue => f.write_str("missing value"),
            ParseError::InvalidValue(e) => write!(f, "invalid value: {}", e),
        }
    }
}

impl FromStr for Field {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut components = s.splitn(MAX_COMPONENTS, SEPARATOR);

        let key: Key = components
            .next()
            .ok_or_else(|| ParseError::MissingKey)
            .and_then(|s| s.parse().map_err(ParseError::InvalidKey))?;

        let value = if let Type::Flag = key.ty() {
            let s = components.next().unwrap_or_default();
            Value::from_str_key(s, &key).map_err(ParseError::InvalidValue)?
        } else {
            components
                .next()
                .ok_or_else(|| ParseError::MissingValue)
                .and_then(|s| Value::from_str_key(s, &key).map_err(ParseError::InvalidValue))?
        };

        Ok(Self::new(key, value))
    }
}

#[cfg(test)]
mod tests {
    use crate::header::Number;

    use super::*;

    #[test]
    fn test_from_str() -> Result<(), ParseError> {
        let actual: Field = "NS=2".parse()?;
        assert_eq!(actual.key(), &Key::SamplesWithDataCount);
        assert_eq!(actual.value(), &Value::Integer(2));

        let actual: Field = "BQ=1.333".parse()?;
        assert_eq!(actual.key(), &Key::BaseQuality);
        assert_eq!(actual.value(), &Value::Float(1.333));

        let actual: Field = "SOMATIC".parse()?;
        assert_eq!(actual.key(), &Key::IsSomaticMutation);
        assert_eq!(actual.value(), &Value::Flag(true));

        let actual: Field = "SVTYPE=DEL".parse()?;
        assert_eq!(
            actual.key(),
            &Key::Other(String::from("SVTYPE"), Number::Count(1), Type::String)
        );
        assert_eq!(actual.value(), &Value::String(String::from("DEL")));

        Ok(())
    }
}