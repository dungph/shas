use minicbor::{
    data::Type,
    decode::{self, Decoder},
    encode::{self, Encode, Encoder},
};
use serde_json::{Map, Value};

pub fn decode_cbor<'b>(d: &mut Decoder<'b>) -> Result<Value, decode::Error> {
    Ok(match d.datatype()? {
        minicbor::data::Type::Bool => Value::from(d.bool()?),
        minicbor::data::Type::Null => Value::Null,
        minicbor::data::Type::Undefined => Value::Null,
        minicbor::data::Type::U8 => Value::from(d.u8()?),
        minicbor::data::Type::U16 => Value::from(d.u16()?),
        minicbor::data::Type::U32 => Value::from(d.u32()?),
        minicbor::data::Type::U64 => Value::from(d.u64()?),
        minicbor::data::Type::I8 => Value::from(d.i8()?),
        minicbor::data::Type::I16 => Value::from(d.i16()?),
        minicbor::data::Type::I32 => Value::from(d.i32()?),
        minicbor::data::Type::I64 => Value::from(d.i64()?),
        minicbor::data::Type::F16 => Value::from(d.f16()?),
        minicbor::data::Type::F32 => Value::from(d.f16()?),
        minicbor::data::Type::F64 => Value::from(d.f64()?),
        minicbor::data::Type::Simple => Err(decode::Error::Message("Simple not supported"))?,
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
        minicbor::data::Type::Array => Value::from({
            let len = d.array()?.unwrap();
            let mut result = Vec::new();
            for _ in 0..len {
                result.push(decode_cbor(d)?);
            }
            result
        }),
        minicbor::data::Type::ArrayIndef => Value::from({
            let _ = d.array()?;
            let mut result = Vec::new();
            while d.datatype()? != Type::Break {
                result.push(decode_cbor(d)?);
            }
            d.skip()?;
            result
        }),
        minicbor::data::Type::Map => Value::Object({
            let mut result = Map::new();
            let len = d.map()?.unwrap();
            for _ in 0..len {
                if let Some(s) = decode_cbor(d)?.as_str() {
                    result.insert(s.to_owned(), decode_cbor(d)?);
                }
            }
            result
        }),
        minicbor::data::Type::MapIndef => Value::Object({
            let mut result = Map::new();
            let _ = d.map()?;
            while d.datatype()? != Type::Break {
                if let Some(s) = decode_cbor(d)?.as_str() {
                    result.insert(s.to_owned(), decode_cbor(d)?);
                }
            }
            Decoder::skip(d)?;
            result
        }),
        minicbor::data::Type::Tag => Err(decode::Error::Message("Tag not supported"))?,
        minicbor::data::Type::Break => Err(decode::Error::Message("Second break"))?,
        minicbor::data::Type::Unknown(n) => Err(decode::Error::UnknownVariant(n.into()))?,
    })
}

pub fn encode_cbor<W: encode::Write>(
    value: &Value,
    e: &mut Encoder<W>,
) -> Result<(), encode::Error<W::Error>> {
    match value {
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
                encode_cbor(element, e)?;
            }
        }
        Value::Object(o) => {
            e.map(o.len() as u64)?;
            for (k, v) in o {
                k.encode(e)?;
                encode_cbor(v, e)?;
            }
        }
    };
    Ok(())
}

#[test]
fn test_json_cbor() {
    let value = serde_json::json!({
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

    let mut encoder = Encoder::new(&mut buf[..]);
    encode_cbor(&value, &mut encoder).unwrap();
    let written = encoder.into_inner().as_ref() as *const [u8] as *const () as usize
        - &buf as *const [u8] as *const () as usize;

    let mut decoder = Decoder::new(&buf[..written]);

    let svalue: Value = decode_cbor(&mut decoder).unwrap();
    assert_eq!(value, svalue);
}
