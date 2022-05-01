use std::borrow::Cow;
use std::collections::HashMap;

use crate::{error::{Error, ConversionError}, value::{Value, Node, Relationship, UnboundRelationship}};
use serde::{forward_to_deserialize_any, serde_if_integer128};
use serde::de::{
    self, Deserializer, Visitor, Expected, Unexpected, DeserializeOwned, SeqAccess, IntoDeserializer, DeserializeSeed, MapAccess
};

pub fn from_value<T>(value: Value) -> Result<T, Error>
    where T: DeserializeOwned,
{
    T::deserialize(value)
}

impl Value {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
        where E: serde::de::Error
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected<'_> {
        match *self {
            // V1-compatible value types
            Value::Null => Unexpected::Unit,
            Value::Boolean(b) => Unexpected::Bool(b),
            Value::Integer(n) => Unexpected::Signed(n),
            Value::Float(f) => Unexpected::Float(f),
            Value::List(_) => Unexpected::Seq,
            Value::Map(_) => Unexpected::Map,
            Value::Node(_) => Unexpected::Other("Node"),
            Value::Path(_) => Unexpected::Other("Path"),
            Value::Relationship(_) => Unexpected::Other("Relationship"),
            Value::Bytes(ref b) => Unexpected::Bytes(b),
            Value::String(ref s) => Unexpected::Str(s),
            Value::UnboundRelationship(_) => Unexpected::Other("UnboundRelationship"),

            // V2+-compatible value types
            Value::Date(_) => Unexpected::Other("NaiveDate"), // A date without a time zone, i.e. LocalDate
            Value::Time(_, _) => Unexpected::Other("OffsetTime"), // A time with UTC offset, i.e. OffsetTime
            Value::DateTimeOffset(_) => Unexpected::Other("OffsetDateTime"), // A date-time with UTC offset, i.e. OffsetDateTime
            Value::DateTimeZoned(_) => Unexpected::Other("ZonedDateTime"),  // A date-time with time zone ID, i.e. ZonedDateTime
            Value::LocalTime(_) => Unexpected::Other("NaiveTime"),         // A time without time zone
            Value::LocalDateTime(_) => Unexpected::Other("NaiveDateTime"), // A date-time without time zone
            Value::Duration(_) => Unexpected::Other("Duration"),
            Value::Point2D(_) => Unexpected::Other("Point2D"),
            Value::Point3D(_) => Unexpected::Other("Point3D"),
        }
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self {
            // V1-compatible value types
            Value::Null => visitor.visit_unit(),
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Integer(n) => visitor.visit_i64(n),
            Value::Float(f) => visitor.visit_f64(f),
            Value::String(s) => visitor.visit_string(s),
            Value::List(l) => visit_list(l, visitor),
            Value::Map(m) => visit_map(m, visitor),
            Value::Node(n) => visit_node(n, visitor), // TODO ?
            Value::Relationship(r) => visit_relationship(r, visitor),
            Value::Bytes(ref b) => visitor.visit_bytes(b),
            Value::UnboundRelationship(r) => visit_unbound_relationship(r, visitor),

            // // V2+-compatible value types
            // Value::Date(_) => Unexpected::Other("NaiveDate"), // A date without a time zone, i.e. LocalDate
            // Value::Time(_, _) => Unexpected::Other("OffsetTime"), // A time with UTC offset, i.e. OffsetTime
            // Value::DateTimeOffset(_) => Unexpected::Other("OffsetDateTime"), // A date-time with UTC offset, i.e. OffsetDateTime
            // Value::DateTimeZoned(_) => Unexpected::Other("ZonedDateTime"),  // A date-time with time zone ID, i.e. ZonedDateTime
            // Value::LocalTime(_) => Unexpected::Other("NaiveTime"),         // A time without time zone
            // Value::LocalDateTime(_) => Unexpected::Other("NaiveDateTime"), // A date-time without time zone
            // Value::Duration(_) => Unexpected::Other("Duration"),
            // Value::Point2D(_) => Unexpected::Other("Point2D"),
            // Value::Point3D(_) => Unexpected::Other("Point3D"),
            _ => Err(Error::ConversionError(ConversionError::Serde(self.unexpected().to_string()))),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct ignored_any
        unit_struct tuple_struct tuple enum identifier
    }
}


// TODO - Use one in serde::de::value?
struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}
struct MapDeserializer {
    iter: <HashMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: HashMap<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Owned(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
        where T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}



struct MapKeyDeserializer<'de> {
    key: Cow<'de, str>,
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
            where V: Visitor<'de>,
        {
            match (self.key.parse(), self.key) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
                (Err(_), Cow::Owned(s)) => visitor.visit_string(s),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>,
    {
        BorrowedCowStrDeserializer::new(self.key).deserialize_any(visitor)
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);

    serde_if_integer128! {
        deserialize_integer_key!(deserialize_i128 => visit_i128);
        deserialize_integer_key!(deserialize_u128 => visit_u128);
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>,
    {
        // Map keys cannot be null
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
        where V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where V: Visitor<'de>,
    {
        self.key
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}


struct BorrowedCowStrDeserializer<'de> {
    value: Cow<'de, str>,
}

impl<'de> BorrowedCowStrDeserializer<'de> {
    fn new(value: Cow<'de, str>) -> Self {
        BorrowedCowStrDeserializer { value }
    }
}

impl<'de> de::Deserializer<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
        where V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> de::EnumAccess<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = Error;
    type Variant = UnitOnly;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Error>
        where T: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self)?;
        Ok((value, UnitOnly))
    }
}

struct UnitOnly;

impl<'de> de::VariantAccess<'de> for UnitOnly {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Error>
            where T: de::DeserializeSeed<'de>, {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor<'de>, {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
        where V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}


fn visit_list<'de, V>(array: Vec<Value>, visitor: V) -> Result<V::Value, Error>
    where V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_map<'de, V>(object: HashMap<String, Value>, visitor: V) -> Result<V::Value, Error>
    where V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

fn visit_node<'de, V>(node: Node, visitor: V) -> Result<V::Value, Error>
    where V: Visitor<'de>,
{
    visit_map([
            ("id".to_string(), Value::from(node.node_identity)),
            ("labels".to_string(), Value::from(node.labels)),
        ].into_iter().chain(
            node.properties
        ).collect(), visitor)
}

fn visit_relationship<'de, V>(rel: Relationship, visitor: V) -> Result<V::Value, Error>
    where V: Visitor<'de>,
{
    visit_map([
            ("id".to_string(), Value::from(rel.rel_identity)),
            ("start_node_id".to_string(), Value::from(rel.start_node_identity)),
            ("end_node_id".to_string(), Value::from(rel.end_node_identity)),
            ("label".to_string(), Value::from(rel.rel_type)),
        ].into_iter().chain(
            rel.properties
        ).collect(), visitor)
}

fn visit_unbound_relationship<'de, V>(rel: UnboundRelationship, visitor: V) -> Result<V::Value, Error>
    where V: Visitor<'de>,
{
    visit_map([
            ("id".to_string(), Value::from(rel.rel_identity)),
            ("label".to_string(), Value::from(rel.rel_type)),
        ].into_iter().chain(
            rel.properties
        ).collect(), visitor)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::value::Node;
    use crate::value::Relationship;
    use crate::value::UnboundRelationship;

    use super::Value;
    use super::from_value;

    use serde::Deserialize;

    #[test]
    fn test_de_bool_tuple() {
        type Data = (bool, bool);
        let expected = (true, false);
        let bolt_value = Value::from(vec![true, false]);
        let value: Data = from_value(bolt_value).unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_de_numbers_tuple() {
        type Data = (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
        let expected = (-1, -2, -4, -8, -16, 1, 2, 4, 8, 16, 0.1, 0.2);
        let bolt_value = Value::List(
            vec![-1, -2, -4, -8, -16, 1, 2, 4, 8, 16].into_iter().map(Value::Integer)
            .chain(vec![0.1, 0.2].into_iter().map(Value::Float))
            .collect()
        );
        let value: Data = from_value(bolt_value).unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_de_string() {
        let expected = "test string!";
        let bolt_value = Value::from(expected);
        let value: String = from_value(bolt_value).unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_de_vec() {
        let expected = vec!["a","b","c"];
        let bolt_value = Value::from(expected.clone());
        let value: Vec<String> = from_value(bolt_value).unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_de_map() {
        let expected = HashMap::from_iter(vec![("a", "b"), ("c", "d")].into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        let bolt_value = Value::from(expected.clone());
        let value: HashMap<String, String> = from_value(bolt_value).unwrap();
        assert_eq!(value, expected);
    }

    #[derive(PartialEq, Debug, Deserialize)]
    struct TestNode {
        id: i64,
        labels: Vec<String>,
        a: String
    }

    #[test]
    fn test_de_node() {
        let expected = TestNode {
            id: 1,
            labels: vec!["test".to_string()],
            a: "b".to_string(),
        };
        let bolt_value = Value::from(Node {
            node_identity: 1,
            labels: vec!["test".to_string()],
            properties: HashMap::from_iter(vec![("a".to_string(), Value::from("b"))]),
        });

        let value: TestNode = from_value(bolt_value).unwrap();

        assert_eq!(value, expected);
    }

    #[derive(PartialEq, Debug, Deserialize)]
    struct TestRelationship {
        id: i64,
        start_node_id: i64,
        end_node_id: i64,
        label: String,
        a: String
    }

    #[test]
    fn test_de_relationship() {
        let expected = TestRelationship {
            id: 1,
            start_node_id: 2,
            end_node_id: 3,
            label: "test".to_string(),
            a: "b".to_string(),
        };
        let bolt_value = Value::from(Relationship {
            rel_identity: 1,
            start_node_identity: 2,
            end_node_identity: 3,
            rel_type: "test".to_string(),
            properties: HashMap::from_iter(vec![("a".to_string(), Value::from("b"))]),
        });

        let value: TestRelationship = from_value(bolt_value).unwrap();

        assert_eq!(value, expected);
    }

    #[derive(PartialEq, Debug, Deserialize)]
    struct TestUnboundRelationship {
        id: i64,
        label: String,
        a: String
    }

    #[test]
    fn test_de_unbound_relationship() {
        let expected = TestUnboundRelationship {
            id: 1,
            label: "test".to_string(),
            a: "b".to_string(),
        };
        let bolt_value = Value::from(UnboundRelationship {
            rel_identity: 1,
            rel_type: "test".to_string(),
            properties: HashMap::from_iter(vec![("a".to_string(), Value::from("b"))]),
        });

        let value: TestUnboundRelationship = from_value(bolt_value).unwrap();

        assert_eq!(value, expected);
    }
}
