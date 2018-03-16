#[cfg(feature = "serialize")]
extern crate serde;

use std::path::PathBuf;
use std::str::FromStr;
use std::fmt::{Display, Formatter, Error as FmtError};


mod parse {
    include!(concat!(env!("OUT_DIR"), "/parse.rs"));
}

pub use self::parse::{parse_uri, ParseError as UriError, ParseResult as UriResult};

fn hex_decode(mut first: u8, mut second: u8) -> Result<u8, &'static str> {
    let zero = '0' as u8;
    first -= zero;
    second -= zero;
    if first >= 16 || second >= 16 {
        return Err("invalid hex digit")
    }
    return Ok(first * 16 + second)
}

fn percent_decode(input: &str) -> Result<String, &'static str>  {
    use std::str::Bytes;

    struct Decoder<'a> {
        by: Bytes<'a>,
    }

    impl<'a> Iterator for Decoder<'a> {
        type Item = Result<u8, &'static str>;

        fn next(&mut self) -> Option<Self::Item> {
            match self.by.next() {
                None => None,
                Some(37 /* % */) => Some(match (self.by.next(), self.by.next()) {
                    (Some(a), Some(b)) => hex_decode(a, b),
                    _ => Err("incomplete hex byte"),
                }),
                Some(c) => Some(Ok(c)),
            }
        }
    }

    let decoded: Result<Vec<u8>, &'static str> = Decoder { by: input.bytes() }
        .collect();
    String::from_utf8(decoded?)
        .map_err(|_| "percent-encoded string is not valid UTF-8")
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Authority {
    pub user: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Host {
    pub name: String,
    pub port: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uri {
    pub scheme: Option<String>,
    pub auth: Option<Authority>,
    pub host: Option<Host>,
    pub path: PathBuf,
    // TODO: # and &
}

impl FromStr for Uri {
    type Err = UriError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_uri(s)
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        // TODO: percent re-encode

        println!("{:?}", self);
        if let Some(ref scheme) = self.scheme {
            write!(f, "{}:", scheme)?;
        }
        if self.scheme.is_some() && self.host.is_some() {
            write!(f, "//")?;
        }
        if let Some(ref auth) = self.auth {
            match auth.password {
                Some(ref pass) => write!(f, "{}:{}@", auth.user, pass),
                None => write!(f, "{}@", auth.user),
            }?;
        }
        if let Some(ref host) = self.host {
            match host.port {
                Some(ref port) => write!(f, "{}:{}", host.name, port),
                None => write!(f, "{}", host.name),
            }?;
        }
        write!(f, "{}", self.path.display())
    }
}

#[cfg(feature = "serialize")]
impl serde::Serialize for Uri {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        format!("{}", self).serialize(ser)
    }
}

#[cfg(feature = "serialize")]
impl<'de> serde::Deserialize<'de> for Uri {
    fn deserialize<D>(de: D) -> Result<Uri, D::Error> where D: serde::Deserializer<'de> {
        use serde::de::{Error as DeError};
        
        let string = String::deserialize(de)?;
        Uri::from_str(&string).map_err(DeError::custom)
    }
}

#[test]
fn test_wiki_examples() {
    let examples = vec![
        "abc://username:password@example.com:123/path/data",
        "https://example.org/absolute/URI/with/absolute/path/to/resource.txt",
        "//example.org/scheme-relative/URI/with/absolute/path/to/resource.txt",
        "//example.org/scheme-relative/URI/with/absolute/path/to/resource",
        "/relative/URI/with/absolute/path/to/resource.txt",
        "relative/path/to/resource.txt",
        "../../../resource.txt",
        //"./resource.txt#frag01",
        "resource.txt",
    ];

    for mut ex in examples {
        let u = Uri::from_str(ex).expect(&format!("Could not parse valid URI: \"{}\"", ex));
        if ex.starts_with("//") {
            ex = &ex[2..];
        }
        assert_eq!(ex, &format!("{}", u));
    }
}
