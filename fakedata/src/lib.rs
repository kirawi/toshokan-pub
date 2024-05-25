use csv::Csv;
use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};

mod csv;

static TITLE_DESC: Lazy<Csv> = Lazy::new(|| {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/works.csv"));
    Csv::from(source)
});

static FIRST_NAME: Lazy<Csv> = Lazy::new(|| {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/first_name.csv"));
    Csv::from(source)
});

static LAST_NAME: Lazy<Csv> = Lazy::new(|| {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/last_name.csv"));
    Csv::from(source)
});

pub fn random_title_desc() -> (String, String) {
    let mut rand = thread_rng();
    let row = TITLE_DESC
        .entries
        .get(rand.gen_range(0..TITLE_DESC.entries.len()))
        .unwrap();
    (row[0].clone(), row[1].clone())
}

pub fn random_name() -> String {
    let mut rand = thread_rng();
    let first = &FIRST_NAME
        .entries
        .get(rand.gen_range(0..FIRST_NAME.entries.len()))
        .unwrap()[0];
    let mut last = LAST_NAME
        .entries
        .get(rand.gen_range(0..LAST_NAME.entries.len()))
        .unwrap()[0]
        .clone();
    last.push_str(first);
    last
}
