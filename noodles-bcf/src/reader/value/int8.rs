#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Int8 {
    Value(i8),
    Missing,
    EndOfVector,
    Reserved(i8),
}

impl From<i8> for Int8 {
    fn from(value: i8) -> Self {
        match value as u8 {
            0x80 => Self::Missing,
            0x81 => Self::EndOfVector,
            0x82..=0x87 => Self::Reserved(value),
            _ => Self::Value(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i8_for_int8() {
        assert_eq!(Int8::from(0), Int8::Value(0));
        assert_eq!(Int8::from(-128), Int8::Missing);
        assert_eq!(Int8::from(-127), Int8::EndOfVector);
        assert_eq!(Int8::from(-126), Int8::Reserved(-126));
        assert_eq!(Int8::from(-125), Int8::Reserved(-125));
        assert_eq!(Int8::from(-124), Int8::Reserved(-124));
        assert_eq!(Int8::from(-123), Int8::Reserved(-123));
        assert_eq!(Int8::from(-122), Int8::Reserved(-122));
        assert_eq!(Int8::from(-121), Int8::Reserved(-121));
    }
}
