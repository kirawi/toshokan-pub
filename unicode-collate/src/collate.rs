use uca_generate::{derive_weight, lookup, parser::CollateElement};

use unicode_canonical_combining_class::get_canonical_combining_class_u32 as ccc;

pub struct Key {
    /*
        Per the standard, a sort key is formed by sequentially iterating over every collation element
        for each collation level and appending any non-zero weight. A 0 is appended after each non-primary
        level.
    */
    inner: Vec<u16>,
}

pub struct CEArray {
    pub inner: Vec<CollateElement>,
}

/*
    1 If `src` is a single char, just return the lookup for that
    2. Find the longest matching string
        2.1 First find the longest contiguous string via lookup for the next `n` characters.

*/
// Src is normalized already
pub fn find_cea(src: &mut Vec<u32>) -> CEArray {
    // Longest by default
    if src.len() == 1 {
        src.pop();
        return CEArray {
            inner: derive_weight(src[0]),
        };
    }

    let mut buf = vec![];
    let mut i = 0;

    // Contiguous match w/ only starters
    while i < src.len() {
        // Non-starter
        // TODO Should prob handle a missing starter
        if ccc(src[i]) as u8 == 0 {
            // We do not try discontig matching on starters
            let Some(ce) = lookup(&[buf.as_slice(), &[src[i]]].concat()) else {
                return lookup;
            };
        }
    }

    // Non-starter time
    let ccc_b = ccc(src[1]) as u8;
    let blocking = src.len() >= 3
        && (matches!(src[1], 173 | 847) || (ccc_b == 0 || ccc_b >= ccc(src[2]) as u8));

    todo!();
}
