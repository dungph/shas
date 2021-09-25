use std::collections::HashMap;

use minicbor::{
    decode::{self, Decode, Decoder},
    encode::{self, Encode, Encoder},
};
use serde_json::Number;

pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(&s),
            _ => None,
        }
    }
    fn into_json(self) -> serde_json::Value {
        self.into()
    }
    fn from_json(j: serde_json::Value) -> Self {
        j.into()
    }
}

impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(n),
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(a) => Value::Array(a.into_iter().map(|v| v.into()).collect()),
            serde_json::Value::Object(obj) => {
                Value::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

impl Into<serde_json::Value> for Value {
    fn into(self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::from(b),
            Value::Number(n) => serde_json::Value::from(n),
            Value::String(s) => serde_json::Value::from(s),
            Value::Array(a) => serde_json::Value::from(a),
            Value::Object(o) => {
                serde_json::Value::Object(o.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

impl Decode<'_> for Value {
    fn decode<'b>(d: &mut Decoder<'b>) -> Result<Self, decode::Error> {
        Ok(match d.datatype()? {
            minicbor::data::Type::Bool => Value::Bool(d.bool()?),
            minicbor::data::Type::Null => Value::Null,
            minicbor::data::Type::Undefined => Value::Null,
            minicbor::data::Type::U8 => Value::Number(d.u8()?.into()),
            minicbor::data::Type::U16 => Value::Number(d.u16()?.into()),
            minicbor::data::Type::U32 => Value::Number(d.u32()?.into()),
            minicbor::data::Type::U64 => Value::Number(d.u64()?.into()),
            minicbor::data::Type::I8 => Value::Number(d.i8()?.into()),
            minicbor::data::Type::I16 => Value::Number(d.i16()?.into()),
            minicbor::data::Type::I32 => Value::Number(d.i32()?.into()),
            minicbor::data::Type::I64 => Value::Number(d.i64()?.into()),
            minicbor::data::Type::F16 => Value::Number(
                Number::from_f64(d.f16()?.into()).ok_or(decode::Error::Message("NaN"))?,
            ),
            minicbor::data::Type::F32 => Value::Number(
                Number::from_f64(d.f16()?.into()).ok_or(decode::Error::Message("NaN"))?,
            ),
            minicbor::data::Type::F64 => Value::Number(
                Number::from_f64(d.f16()?.into()).ok_or(decode::Error::Message("NaN"))?,
            ),
            minicbor::data::Type::Simple => todo!(),
            minicbor::data::Type::Bytes => {
                let mut buf = String::from("#");
                let bytes = d.bytes()?;
                base64::encode_config_buf(bytes, base64::STANDARD, &mut buf);
                Value::String(buf)
            }
            minicbor::data::Type::BytesIndef => {
                let mut buf = String::from("#");
                let mut bytes = Vec::new();
                d.bytes_iter()?
                    .collect::<Result<Vec<&[u8]>, decode::Error>>()?
                    .iter()
                    .for_each(|bs| bytes.extend_from_slice(bs));
                base64::encode_config_buf(bytes, base64::STANDARD, &mut buf);
                Value::String(buf)
            }
            minicbor::data::Type::String => Value::String(d.str()?.into()),
            minicbor::data::Type::StringIndef => {
                let mut buf = String::new();
                d.str_iter()?
                    .collect::<Result<Vec<&str>, decode::Error>>()?
                    .iter()
                    .for_each(|s| buf.push_str(s));
                Value::String(buf)
            }
            minicbor::data::Type::Array => Value::Array(
                d.array_iter()?
                    .collect::<Result<Vec<Value>, decode::Error>>()?,
            ),
            minicbor::data::Type::ArrayIndef => Value::Array(
                d.array_iter()?
                    .collect::<Result<Vec<Value>, decode::Error>>()?,
            ),
            minicbor::data::Type::Map => Value::Object(
                d.map_iter()?
                    .collect::<Result<Vec<(Value, Value)>, decode::Error>>()?
                    .into_iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_owned(), v))) //wk.value.as_str().map(|s| (s.to_owned(), wv.value)))
                    .collect::<HashMap<String, Value>>(),
            ),
            minicbor::data::Type::MapIndef => Value::Object(
                d.map_iter()?
                    .collect::<Result<Vec<(Value, Value)>, decode::Error>>()?
                    .into_iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_owned(), v)))
                    .collect::<HashMap<String, Value>>(),
            ),
            minicbor::data::Type::Tag => todo!(),
            minicbor::data::Type::Break => todo!(),
            minicbor::data::Type::Unknown(_) => todo!(),
        })
    }
}

impl Encode for Value {
    fn encode<W: encode::Write>(&self, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>> {
        match self {
            Value::Null => {
                e.null()?;
            }
            Value::Bool(b) => {
                e.bool(*b)?;
            }
            Value::Number(n) => {
                if let Some(n) = n.as_f64() {
                    e.f64(n)?;
                } else if let Some(n) = n.as_i64() {
                    e.i64(n)?;
                } else {
                    e.u64(n.as_u64().unwrap())?;
                }
            }
            Value::String(s) => {
                if s.starts_with('#') {
                    match base64::decode(s.split_once('#').unwrap().1) {
                        Ok(bytes) => e.bytes(&bytes)?,
                        Err(_) => e.str(s)?,
                    };
                } else {
                    e.str(s)?;
                }
            }
            Value::Array(a) => {
                e.array(a.len() as u64)?;
                for element in a {
                    element.encode(e)?;
                }
            }
            Value::Object(o) => {
                e.map(o.len() as u64)?;
                for (k, v) in o {
                    k.encode(e)?;
                    v.encode(e)?;
                }
            }
        };
        Ok(())
    }
}

#[test]
fn test_json_cbor() {
    let json_value = serde_json::json!({
       "name": "Pha",
       "map": {
           "k1": "v1",
           "k2": {
               "k1": "v1",
               "k2": "v2"
           },
           "k3": ["a", "b", "c"],
       }
    });

    let mut buf = [0u8; 100];

    let value: Value = json_value.clone().into();
    let mut encoder = Encoder::new(&mut buf[..]);
    value.encode(&mut encoder).unwrap();
    let written = encoder.into_inner().as_ref() as *const [u8] as *const () as usize
        - &buf as *const [u8] as *const () as usize;

    let mut decoder = Decoder::new(&buf[..written]);

    let svalue: Value = decoder.decode().unwrap();
    let json2: serde_json::Value = svalue.into();
    assert_eq!(json_value, json2);
}
