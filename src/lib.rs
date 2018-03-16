use std::path::PathBuf;
use std::str::FromStr;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authority {
    pub user: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    pub name: String,
    pub port: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[test]
fn test_wiki_examples() {
    let examples = vec![
        "https://example.org/absolute/URI/with/absolute/path/to/resource.txt",
        "//example.org/scheme-relative/URI/with/absolute/path/to/resource.txt",
        "//example.org/scheme-relative/URI/with/absolute/path/to/resource",
        "/relative/URI/with/absolute/path/to/resource.txt",
        "relative/path/to/resource.txt",
        "../../../resource.txt",
        //"./resource.txt#frag01",
        "resource.txt",
    ];

    for ex in examples {
        Uri::from_str(ex).expect(&format!("Could not parse valid URI: \"{}\"", ex));
    }
}
