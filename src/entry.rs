//! Helper types

use std::time::SystemTime;

use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
use fakedata::{random_name, random_title_desc};
use rand::Rng;
use serde::Serialize;
use uuid::Uuid;

use crate::{stats::Statistics, user::UserRef};

/// A written Work (novel/comic/etc.). It can contain text and images
// TODO: See if this is even possible w/ borrow checker and sled's db
#[derive(Serialize, Encode, Decode)]
pub struct LiteraryWork {
    pub title: String,
    pub description: String,
    pub chapters: Vec<Chapter>,

    pub creators: Vec<UserRef>,
    pub tags: Vec<Tag>,

    // Dates
    pub publish: SystemTime,
    pub update: SystemTime,

    pub stats: Statistics,
}

#[derive(Serialize, Encode, Decode)]
pub enum Tag {
    Genre(String),
    Other(String),
}

/// A Comic can be represented via a 1-page chapter
// TODO: Pages may not matter depend on how I choose to render it (all on one page, or actually split it by pages?)
#[derive(Serialize, Encode, Decode)]
pub struct Chapter {
    // TODO: Check if UUID is being handled right
    #[bincode(with_serde)]
    pub id: Uuid,
    pub title: String,
    pub elements: Vec<Entry>,
    #[bincode(with_serde)]
    pub date: DateTime<Utc>,
}

/// Rendered in a sequence; no fancy formatting
#[derive(Serialize, Encode, Decode)]
pub enum Entry {
    Paragraph(String),
    // TODO: This may be compressed data, so tbd
    Image(Vec<u8>),
}

pub fn create_rand_work() -> LiteraryWork {
    let mut rng = rand::thread_rng();
    let (title, description) = random_title_desc();

    // TODO: First need to change User to be keyed by UUID
    let creators: Vec<_> = (1..=rng.gen_range(1..=3))
        .map(|_| UserRef {
            name: random_name(),
            created: SystemTime::now(),
        })
        .collect();

    let chapters = (1..=rng.gen_range(1..100)).map(|i| {
        let elements: Vec<_> = (1..=rng.gen_range(1..30)).map(|_| {
            let s = "証ケオヨホ売4面ヨツサリ教家ク供哲目いッご朝育えず頭高イで込月メラロ理新スト木使やむんば日月5創船断おちもき。友ソヤナ表申ひはでろ刊不滅え探剤リて到法ムケナユ率者や障婚んぞれ北7太場レ著保で文提手ワヒヱメ無匹恒めのざほ。討興ネチ元9豊ニカ億張すてぼぜ埋野舗ぼこづは料読キヲマ反8梨ぶ宮吉ぐごょフ爺聞華ヤヱム滋極たクわ一携ヤサワテ供著近種だねど。";
            Entry::Paragraph(s.to_string())
        }).collect();
        Chapter {
            id: Uuid::now_v7(),
            title: format!("Chapter {i}"),
            elements,
            date: Utc::now(),
        }
    }).collect();

    LiteraryWork {
        title,
        description,
        chapters,
        creators,
        tags: vec![],
        publish: SystemTime::now(),
        update: SystemTime::now(),
    }
}
