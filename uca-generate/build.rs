#[path = "src/parser.rs"]
mod parser;

use parser::*;
use std::io::{Read, Write as _};
use std::path::PathBuf;

struct Parser<R: Read> {
    inner: LineParser<R>,
    implicit_weights: Vec<ImplicitWeight>,
    entries: Vec<Entry>,
}

impl<R: Read> Parser<R> {
    fn new(r: R) -> Self {
        Self {
            inner: LineParser::new(r),
            implicit_weights: Vec::default(),
            entries: Vec::default(),
        }
    }

    fn parse(mut self) -> std::io::Result<Table> {
        loop {
            match self.inner.parse_line()? {
                Kind::Version(_) => {}
                Kind::ImplicitWeight(weight) => self.implicit_weights.push(weight),
                Kind::Entry(entry) => self.entries.push(entry),
                Kind::EOF => break,
            }
        }
        self.implicit_weights
            .sort_unstable_by(|a, b| a.range.clone().partial_cmp(b.range.clone()).unwrap());
        self.entries
            .sort_unstable_by(|a, b| a.codepoints.partial_cmp(&b.codepoints).unwrap());
        Ok(Table {
            implicit_weights: self.implicit_weights,
            entries: self.entries,
        })
    }
}

struct Table {
    implicit_weights: Vec<ImplicitWeight>,
    entries: Vec<Entry>,
}

impl Table {
    fn content(&self, p: PathBuf) -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(p)?;
        let max_size = self
            .entries
            .iter()
            .max_by(|a, other| a.codepoints.len().cmp(&other.codepoints.len()))
            .unwrap()
            .codepoints
            .len();

        writeln!(f, "pub const MAX_SIZE: usize = {max_size};")?;
        writeln!(
            f,
            "pub const IMPLICIT_WEIGHTS: &[(std::ops::Range<u32>, u16)] = &[{}];",
            self.implicit_weights
                .iter()
                .map(|iw| format!("({}..{}, {}),\n", iw.range.start, iw.range.end, iw.weight))
                .collect::<String>()
        )?;

        writeln!(
            f,
            "pub const ENTRIES: &[(&[u32], &[[u16; 3]])] = &[{}];",
            self.entries
                .iter()
                .map(|e| {
                    format!(
                        "(&[{}], &[{}]),\n",
                        e.codepoints
                            .iter()
                            .map(|n| format!("{n},"))
                            .collect::<String>(),
                        e.collate_elements
                            .iter()
                            .cloned()
                            .map(|a| format!("[{}, {}, {}],", a[0], a[1], a[2]))
                            .collect::<String>()
                    )
                })
                .collect::<String>()
        )?;

        Ok(())
    }
}

fn main() {
    let p = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("table.rs");
    let allkeys_p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/15-1-allkeys.txt");
    let f = std::fs::File::open(allkeys_p).unwrap();
    let parser = Parser::new(f);
    let table = parser.parse().unwrap();
    table.content(p).unwrap();
}
