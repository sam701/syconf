use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
use std::vec;

use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};

pub use func::Function;
pub use syconf_lib::ErrorWithLocation;
use syconf_lib::{Func, Value, ValueString};

use crate::Error;

mod func;
#[cfg(test)]
mod tests;

thread_local! {
    static FUNC: RefCell<Option<Func>> = RefCell::new(None);
}

pub struct Deserializer {
    value: Value,
}

impl Deserializer {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Bool(x) => visitor.visit_bool(x),
            Value::Int(x) => visitor.visit_i32(x),
            Value::String(x) => visitor.visit_string(x.to_string()),
            Value::HashMap(_) => self.deserialize_map(visitor),
            Value::List(_) => self.deserialize_seq(visitor),
            Value::Func(_) => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.value.as_bool()?)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.value.as_int()?)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        _visitor.visit_char(
            self.value
                .as_value_string()?
                .chars()
                .next()
                .ok_or_else(|| Error::Custom("empty string".to_owned()))?,
        )
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.value.as_value_string()?.to_string())
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // TODO: decode base64
        Err(Error::UnsupportedType)
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Func(func) = &self.value {
            FUNC.with(|x| {
                let mut opt = x.borrow_mut();
                *opt = Some(func.clone());
            });
        }
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Seq {
            iterator: self.value.as_list()?.to_vec().into_iter(),
        })
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map::new(self.value.as_hashmap()?.clone()))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(Enum(self.value))
    }

    fn deserialize_identifier<V>(
        self,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(
        self,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct Map {
    iterator: hash_map::IntoIter<ValueString, Value>,
    current: Option<(ValueString, Value)>,
}

impl Map {
    fn new(hm: HashMap<ValueString, Value>) -> Self {
        Self {
            iterator: hm.into_iter(),
            current: None,
        }
    }
}

impl<'de> MapAccess<'de> for Map {
    type Error = Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> Result<Option<<K as DeserializeSeed<'de>>::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.current = self.iterator.next();
        self.current
            .as_ref()
            .map(|(k, _v)| Deserializer::new(Value::String(k.clone())))
            .map(|x| seed.deserialize(x))
            .map_or(Ok(None), |x| x.map(Some))
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> Result<<V as DeserializeSeed<'de>>::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.current
            .as_ref()
            .map(|(_k, v)| Deserializer::new(v.clone()))
            .map(|x| seed.deserialize(x))
            .expect("value")
    }
}

struct Seq {
    iterator: vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for Seq {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<<T as DeserializeSeed<'de>>::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iterator.next() {
            Some(x) => seed.deserialize(Deserializer::new(x)).map(Some),
            None => Ok(None),
        }
    }
}

struct Enum(Value);

impl<'de> EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(<V as DeserializeSeed<'de>>::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let (key, value) = self.0.as_hashmap()?.iter().next().ok_or_else(|| {
            Error::Custom("Cannot deserialize enum because the hashmap is empty".to_owned())
        })?;
        Ok((
            seed.deserialize(Deserializer::new(Value::String(key.clone())))?,
            Enum(value.clone()),
        ))
    }
}

impl<'de> VariantAccess<'de> for Enum {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(
        self,
        seed: T,
    ) -> Result<<T as DeserializeSeed<'de>>::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer::new(self.0))
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(Deserializer::new(self.0), visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(Deserializer::new(self.0), visitor)
    }
}
