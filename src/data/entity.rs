use crate::error::EntityParseError;
use crate::EntityProp;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use vdf_reader::entry::Entry;
use vdf_reader::VdfError;

#[derive(Clone)]
pub struct Entities {
    pub entities: String,
}

impl fmt::Debug for Entities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Entities<'a> {
            #[allow(dead_code)]
            entities: Vec<RawEntity<'a>>,
        }

        Entities {
            entities: self.iter().collect(),
        }
        .fmt(f)
    }
}
pub struct EntitiesIter<'a> {
    buf: &'a str,
}

impl<'a> Iterator for EntitiesIter<'a> {
    type Item = RawEntity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.buf.find('{')?;
        let end = start + self.buf[start..].find('}')?;

        let out = &self.buf[start..end + 1];

        self.buf = &self.buf[end + 1..];

        Some(RawEntity { buf: out })
    }
}

impl<'a> IntoIterator for &'a Entities {
    type Item = RawEntity<'a>;

    type IntoIter = EntitiesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Entities {
    pub fn iter(&self) -> EntitiesIter {
        EntitiesIter {
            buf: &self.entities,
        }
    }
}

#[derive(Clone)]
pub struct RawEntity<'a> {
    buf: &'a str,
}

impl fmt::Debug for RawEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::collections::HashMap;

        self.properties().collect::<HashMap<_, _>>().fmt(f)
    }
}

impl<'a> RawEntity<'a> {
    pub fn as_str(&self) -> &'a str {
        self.buf
    }

    pub fn properties(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = (&'a str, &'a str);

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('"')? + 1;
                let end = start + self.buf[start..].find('"')?;

                let key = &self.buf[start..end];

                let rest = &self.buf[end + 1..];

                let start = rest.find('"')? + 1;
                let end = start + rest[start..].find('"')?;

                let value = &rest[start..end];

                self.buf = &rest[end + 1..];

                Some((key, value))
            }
        }

        Iter { buf: self.buf }
    }

    pub fn prop(&self, key: &'static str) -> Option<&'a str> {
        self.properties()
            .find_map(|(prop_key, value)| (key == prop_key).then_some(value))
    }

    pub fn prop_parse<T: EntityProp<'a>>(
        &self,
        key: &'static str,
    ) -> Option<Result<T, EntityParseError>> {
        self.prop(key).map(T::parse)
    }

    pub fn parse<E: Deserialize<'a>>(&self) -> Result<E, VdfError> {
        vdf_reader::from_str(self.buf)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GenericEntity {
    #[serde(rename = "classname")]
    pub class: String,
    #[serde(flatten)]
    pub data: HashMap<String, Entry>,
}
