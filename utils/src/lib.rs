use minicbor::{
    decode::{self, Tokenizer},
    encode::{self, Encode, Encoder},
};
use serde_json::{Map, Value};

pub fn decode_cbor<'b>(tokenizer: &mut Tokenizer<'b>) -> Result<Value, decode::Error> {
    Ok(match tokenizer.token()? {
        decode::Token::Bool(b) => Value::from(b),
        decode::Token::U8(n) => Value::from(n),
        decode::Token::U16(n) => Value::from(n),
        decode::Token::U32(n) => Value::from(n),
        decode::Token::U64(n) => Value::from(n),
        decode::Token::I8(n) => Value::from(n),
        decode::Token::I16(n) => Value::from(n),
        decode::Token::I32(n) => Value::from(n),
        decode::Token::I64(n) => Value::from(n),
        decode::Token::F16(n) => Value::from(n),
        decode::Token::F32(n) => Value::from(n),
        decode::Token::F64(n) => Value::from(n),
        decode::Token::Bytes(b) => {
            let mut buf = String::from("#");
            base64::encode_config_buf(b, base64::STANDARD, &mut buf);
            Value::String(buf)
        }
        decode::Token::String(s) => Value::from(s),
        decode::Token::Array(n) => {
            let mut result = Vec::new();
            for _ in 0..n {
                result.push(decode_cbor(tokenizer)?);
            }
            Value::from(result)
        }
        decode::Token::Map(m) => {
            let mut result = Map::new();
            for _ in 0..m {
                if let Some(s) = decode_cbor(tokenizer)?.as_str() {
                    result.insert(s.to_owned(), decode_cbor(tokenizer)?);
                }
            }
            Value::from(result)
        }
        decode::Token::Tag(_t) => Err(decode::Error::Message("Tag not yet supported"))?,
        decode::Token::Simple(_s) => Err(decode::Error::Message("Simple not yet supported"))?,
        decode::Token::Break => Err(decode::Error::Message("unexpected break"))?,
        decode::Token::Null => Value::Null,
        decode::Token::Undefined => Value::Null,
        decode::Token::BeginBytes => {
            let mut bytes = Vec::new();
            loop {
                let token = tokenizer.token()?;
                if token == decode::Token::Break {
                    break;
                }
                if let decode::Token::Bytes(b) = token {
                    bytes.extend_from_slice(b);
                }
            }
            let mut buf = String::from("#");
            base64::encode_config_buf(bytes, base64::STANDARD, &mut buf);
            Value::String(buf)
        }
        decode::Token::BeginString => {
            let mut buf = String::new();
            loop {
                let token = tokenizer.token()?;
                if token == decode::Token::Break {
                    break;
                }
                if let decode::Token::String(s) = token {
                    buf.push_str(s);
                }
            }
            Value::String(buf)
        }
        decode::Token::BeginArray => {
            let mut buf = Vec::new();
            loop {
                let mut look_ahead = tokenizer.clone();
                let token = look_ahead.token()?;
                if token == decode::Token::Break {
                    tokenizer.token()?;
                    break;
                } else {
                    buf.push(decode_cbor(tokenizer)?)
                }
            }
            Value::Array(buf)
        }
        decode::Token::BeginMap => {
            let mut buf = Map::new();
            loop {
                let mut look_ahead = tokenizer.clone();
                let token = look_ahead.token()?;
                if token == decode::Token::Break {
                    tokenizer.token()?;
                    break;
                } else {
                    if let Some(s) = decode_cbor(tokenizer)?.as_str() {
                        buf.insert(s.to_owned(), decode_cbor(tokenizer)?);
                    }
                }
            }
            Value::Object(buf)
        }
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
       },
       "k": "#123aA"
    });

    let mut buf = [0u8; 100];

    let mut encoder = Encoder::new(&mut buf[..]);
    encode_cbor(&value, &mut encoder).unwrap();
    let written = encoder.into_inner().as_ref() as *const [u8] as *const () as usize
        - &buf as *const [u8] as *const () as usize;

    let mut decoder = Tokenizer::new(&buf[..written]);

    let svalue: Value = decode_cbor(&mut decoder).unwrap();
    assert_eq!(value, svalue);
}

#[test]
fn test_decode_cbor() {
    let mut buf = [0u8; 1000];
    let begin = &buf as *const _ as *const () as usize;
    let mut encoder = Encoder::new(&mut buf[..]);

    encoder
        .begin_map()
        .unwrap()
        .str("hello")
        .unwrap()
        .begin_str()
        .unwrap()
        .str("ww")
        .unwrap()
        .str("w")
        .unwrap()
        .str("333")
        .unwrap()
        .end()
        .unwrap()
        .str("aa")
        .unwrap()
        .begin_array()
        .unwrap()
        .str("aa")
        .unwrap()
        .str("cc")
        .unwrap()
        .str("bb")
        .unwrap()
        .end()
        .unwrap()
        .end()
        .unwrap();
    let end = encoder.into_inner() as *const _ as *const () as usize;

    let len = end - begin;
    let mut decoder = Tokenizer::new(&buf[..len]);
    let result = decode_cbor(&mut decoder).unwrap();
    let expected = serde_json::json!({
        "hello": "www333",
        "aa": ["aa", "cc", "bb"]
    });
    assert_eq!(result, expected);
}
