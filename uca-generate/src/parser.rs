use std::{
    io::{BufRead, BufReader, Read, Result},
    ops::Range,
};

const VERSION: &'static str = "@version";
const IMPLICIT_WEIGHTS: &'static str = "@implicitweights";

#[derive(Debug)]
#[allow(dead_code)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub variant: usize,
}

pub type CollateElement = [u16; 3];

#[derive(Debug)]
pub struct Entry {
    pub codepoints: Box<[u32]>,
    // Hardcoded to 3 levels
    pub collate_elements: Box<[Box<CollateElement>]>,
}

#[derive(Debug)]
pub struct ImplicitWeight {
    pub range: Range<u32>,
    pub weight: u16,
}

#[derive(Debug)]
pub enum Kind {
    #[allow(dead_code)]
    Version(Version),
    ImplicitWeight(ImplicitWeight),
    Entry(Entry),
    EOF,
}

pub struct LineParser<R: Read>(BufReader<R>);
impl<R: Read> LineParser<R> {
    fn read_line(&mut self, s: &mut String) -> Result<usize> {
        s.clear();

        let n = self.0.read_line(s)?;
        if n == 0 {
            return Ok(0);
        }

        // Remove comments
        if let Some(trimmed) = s
            .split_once(|ch| matches!(ch, '%' | '#'))
            .map(|(res, _)| res)
        {
            s.truncate(trimmed.len());
        }

        Ok(n)
    }

    pub fn parse_line(&mut self) -> Result<Kind> {
        let mut line = String::new();
        loop {
            // EOF
            if self.read_line(&mut line)? == 0 {
                return Ok(Kind::EOF);
            }

            // Empty/Comment line
            if line.trim().is_empty() {
                continue;
            }

            let kind = if line.starts_with(VERSION) {
                parse_version(&line)
            } else if line.starts_with(IMPLICIT_WEIGHTS) {
                parse_implicit_weight(&line)
            } else {
                parse_entry(&line)
            };
            return Ok(kind);
        }
    }

    pub fn new(r: R) -> Self {
        Self(BufReader::new(r))
    }
}

fn parse_version(s: &str) -> Kind {
    let version: Vec<usize> = s[VERSION.len()..]
        .trim()
        .split('.')
        .map(|n| usize::from_str_radix(n, 16).unwrap())
        .collect();
    Kind::Version(Version {
        major: version[0],
        minor: version[1],
        variant: version[2],
    })
}

fn parse_implicit_weight(mut s: &str) -> Kind {
    s = s[IMPLICIT_WEIGHTS.len()..].trim();
    let (range, weight): (String, String) = s
        .split_once(';')
        .map(|(range, weight)| {
            (
                range.chars().filter(|ch| !ch.is_whitespace()).collect(),
                weight.chars().filter(|ch| !ch.is_whitespace()).collect(),
            )
        })
        .unwrap();

    let (start, end) = range
        .split_once("..")
        .map(|(start, end)| {
            (
                u32::from_str_radix(start, 16).unwrap(),
                u32::from_str_radix(end, 16).unwrap(),
            )
        })
        .unwrap();
    let weight = u16::from_str_radix(&weight, 16).unwrap();

    Kind::ImplicitWeight(ImplicitWeight {
        range: start..end,
        weight,
    })
}

fn parse_entry(s: &str) -> Kind {
    let (char_list, mut element_list) = s.split_once(';').unwrap();
    let codepoints = char_list
        .split(|ch: char| ch.is_whitespace())
        .filter(|s| !s.trim().is_empty())
        .map(|n| u32::from_str_radix(n, 16).unwrap())
        .collect::<Vec<_>>();
    let mut collate_elements = Vec::with_capacity(1);
    loop {
        let (offset, element) = parse_collate_element(element_list);

        // Base case
        if offset == element_list.len() {
            break;
        }

        // Handle element
        element_list = &element_list[offset..];
        collate_elements.push(element);
    }

    Kind::Entry(Entry {
        codepoints: codepoints.into_boxed_slice(),
        collate_elements: collate_elements.into_boxed_slice(),
    })
}

fn parse_collate_element(s: &str) -> (usize, Box<[u16; 3]>) {
    let mut weight_buf = Box::new([0; 3]);
    let mut buf = String::with_capacity(5);
    let mut idx = 0;
    let mut byte_offset = 0;

    for ch in s.chars() {
        byte_offset += ch.len_utf8();

        // Base case:
        // - Starting characters
        // - Terminating characters
        // - Weight separators
        // - Ignore whitespace
        match ch {
            '[' => assert_eq!(idx, 0),
            ']' => {
                assert!(!buf.is_empty());
                assert_eq!(idx, 2);
                *weight_buf.get_mut(idx).unwrap_or_else(|| {
                    panic!("Attempted to parse `{s}`");
                }) = u16::from_str_radix(&buf, 16).unwrap_or_else(|_| {
                    panic!("Attempted to parse `{buf}`");
                });
                break;
            }

            // Primary weight
            '.' | '*' if buf.is_empty() => assert_eq!(idx, 0),
            '.' => {
                weight_buf[idx] = u16::from_str_radix(&buf, 16).unwrap_or_else(|_| {
                    panic!("Attempted to parse `{buf}`");
                });
                buf.clear();
                idx += 1;
            }
            ch if ch.is_ascii_alphanumeric() => buf.push(ch),
            _ => {}
        }
    }

    if idx == 0 {
        (s.len(), weight_buf)
    } else {
        (byte_offset, weight_buf)
    }
}
