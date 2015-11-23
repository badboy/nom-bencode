#[macro_use]
extern crate nom;

use std::str;
use std::collections::HashMap;

use nom::IResult;

#[derive(Debug,PartialEq)]
pub enum Value {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<Value>),
    Dict(HashMap<Vec<u8>,Value>)
}

/// parse a u64
fn number(i: &[u8]) -> IResult<&[u8], u64> {
    map_res!(i,
             nom::digit,
             |d| str::FromStr::from_str(str::from_utf8(d).unwrap()))
}

named!(inumber<i64>, chain!(
        pre: opt!(tag!("-")) ~
        n: number
        ,
        || {
            match pre {
                Some(_) => -(n as i64),
                None    => n as i64,
            }
        }
        )
    );

pub fn value(i: &[u8]) -> IResult<&[u8], Value> {
    alt!(i, string | integer | list | dict)
}

fn string(i: &[u8]) -> IResult<&[u8], Value> {
    chain!(i,
           count: number ~
           char!(':') ~
           s: take!(count)
           ,
           || Value::String(s.to_vec())
    )
}

fn integer(i: &[u8]) -> IResult<&[u8], Value> {
    let (i2, n) = try_parse!(i,
                             delimited!(char!('i'), inumber, char!('e'))
                            );

    IResult::Done(i2, Value::Integer(n))
}

fn list(i: &[u8]) -> IResult<&[u8], Value> {
    let (i2, v) = try_parse!(i,
                             delimited!(
                                 char!('l'),
                                 many1!(value),
                                 char!('e')
                                 )
                            );

    IResult::Done(i2, Value::List(v))
}

fn dict(i: &[u8]) -> IResult<&[u8], Value> {
    let (i2, d) = try_parse!(i,
                             delimited!(
                                 char!('d'),
                                 map!(
                                     many1!(pair!(string, value)),
                                     |pairs| {
                                         let mut h = HashMap::<Vec<u8>, Value>::new();
                                         for (key, value) in pairs {
                                             if let Value::String(key) = key {
                                                 h.insert(key, value);
                                             }
                                         }

                                         h
                                     }
                                 ),
                                 char!('e')
                             )
                            );

    IResult::Done(i2, Value::Dict(d))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use super::Value;
    use super::string;
    use super::integer;
    use super::list;
    use super::dict;
    use super::value;

    fn done<T>(x: T) -> ::nom::IResult<&'static [u8], T> {
        ::nom::IResult::Done(&[][..], x)
    }

    fn done_string(x: &[u8]) -> ::nom::IResult<&'static [u8], Value> {
        done(Value::String(x.to_vec()))
    }

    fn done_integer(x: i64) -> ::nom::IResult<&'static [u8], Value> {
        done(Value::Integer(x))
    }

    fn done_list(x: Vec<Value>) -> ::nom::IResult<&'static [u8], Value> {
        done(Value::List(x))
    }

    fn done_dict(x: HashMap<Vec<u8>,Value>) -> ::nom::IResult<&'static [u8], Value> {
        done(Value::Dict(x))
    }

    #[test]
    fn strings() {
        let res = "hello world!".as_bytes();
        let data = "12:hello world!".as_bytes();
        assert_eq!(done_string(res), string(data));
    }

    #[test]
    fn integers() {
        let res = 123;
        let data = "i123e".as_bytes();
        assert_eq!(done_integer(res), integer(data));
    }

    #[test]
    fn negative_integer() {
        let res : i64 = -123;
        let data = "i-123e".as_bytes();
        assert_eq!(done_integer(res), integer(data));
    }

    #[test]
    fn lists() {
        let res = vec![
            Value::String("jelly".as_bytes().to_vec()),
            Value::String("cake".as_bytes().to_vec()),
            Value::String("custard".as_bytes().to_vec())
        ];
        let data = "l5:jelly4:cake7:custarde".as_bytes();
        assert_eq!(done_list(res), list(data));
    }

    #[test]
    fn dicts() {
        let pairs = vec![
            (
                "name".as_bytes().to_vec(),
                Value::String("cream".as_bytes().to_vec())
            ),
            (
                "price".as_bytes().to_vec(),
                Value::Integer(100)
            )
        ];
        let res = HashMap::<Vec<u8>, Value>::from_iter(pairs);
        let data = "d4:name5:cream5:pricei100ee".as_bytes();
        assert_eq!(done_dict(res), dict(data));
    }

    #[test]
    fn larger_example() {
        /*
{
    "foo": 123,
    "bar": {
        "baz": [1, 2, 3, 4, "hello"]
    }
    "qux": "This is a much longer string containing\nnew\nlines\netc.",
    "wibble": [123123, "foo", "bar", "baz", 123124, [1 "a"], ["b" 4], ["cd"]]
}
         */

        let data = b"d3:bard3:bazli1ei2ei3ei4e5:helloee3:fooi123e3:qux54:This is a much longer string containing\nnew\nlines\netc.6:wibbleli123123e3:foo3:bar3:bazi123124eli1e1:ael1:bi4eel2:cdeee";

        let res = value(data);
        assert!(res.is_done());
    }
}

