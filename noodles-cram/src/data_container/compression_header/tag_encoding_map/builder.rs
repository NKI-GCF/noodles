use std::collections::{HashMap, HashSet};

use super::TagEncodingMap;

use crate::{data_container::compression_header::Encoding, record, Record};

#[derive(Debug, Default)]
pub struct Builder {
    keys: HashSet<record::tag::Key>,
}

impl Builder {
    pub fn update(&mut self, record: &Record) {
        for tag in record.tags() {
            self.keys.insert(tag.key());
        }
    }

    pub fn build(self) -> TagEncodingMap {
        let mut map = HashMap::new();

        for key in self.keys {
            let id = key.id();

            let len_encoding = Encoding::External(id);
            let value_encoding = Encoding::External(id);
            let encoding = Encoding::ByteArrayLen(Box::new(len_encoding), Box::new(value_encoding));

            map.insert(id, encoding);
        }

        TagEncodingMap::from(map)
    }
}

#[cfg(test)]
mod tests {
    use noodles_bam::record::data::field::{value::Type, Value};

    use crate::record::{tag::Key, Tag};

    use super::*;

    #[test]
    fn test_build() {
        let nh = Key::new([b'N', b'H'], Type::Int8);
        let co = Key::new([b'C', b'O'], Type::String);

        let mut builder = Builder::default();

        let mut record = Record::default();
        record.tags.push(Tag::new(nh, Value::Int8(1)));
        builder.update(&record);

        let mut record = Record::default();
        record.tags.push(Tag::new(nh, Value::Int8(1)));
        builder.update(&record);

        let mut record = Record::default();
        record
            .tags
            .push(Tag::new(co, Value::String(String::from("noodles"))));
        builder.update(&record);

        let actual = builder.build();

        let expected = vec![
            (
                nh.id(),
                Encoding::ByteArrayLen(
                    Box::new(Encoding::External(nh.id())),
                    Box::new(Encoding::External(nh.id())),
                ),
            ),
            (
                co.id(),
                Encoding::ByteArrayLen(
                    Box::new(Encoding::External(co.id())),
                    Box::new(Encoding::External(co.id())),
                ),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(*actual, expected);
    }
}
