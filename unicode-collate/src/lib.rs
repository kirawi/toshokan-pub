use uca_generate::{lookup, parser::CollateElement, MAX_SIZE};
use unicode_canonical_combining_class::{
    get_canonical_combining_class_u32, CanonicalCombiningClass,
};
use unicode_normalization::UnicodeNormalization;

use std::cmp::Ordering;

// mod collate;

/// Returns the longest contiguous sequence w/ the matched seq
fn find_next_cont_seq(norm: &[u32]) -> Option<(Vec<CollateElement>, Vec<u32>)> {
    // Handle strings that don't fit exactly into the size
    let buf = &norm[..MAX_SIZE.min(norm.len())];

    // Search for longest possible sequence and then reduce down
    for i in (0..buf.len()).rev() {
        if let Some(ce) = lookup(&buf[..=i]) {
            // Offset is incremented if the for loop fails
            return Some((ce, buf[..=i].to_vec()));
        }
    }

    // Handle derived weight
    // HAXX: Not per spec
    Some((vec![[0, 0, 0]], vec![norm[0]]))
}

fn find_next_longest_seq(norm: &mut Vec<u32>) -> Vec<CollateElement> {
    if norm.is_empty() {
        unreachable!();
    }

    // Drain the first n elements corresponding to `s`
    // This function works by just removing the elements from the source string and appending them to `xs`
    let (mut res, mut s) = find_next_cont_seq(norm).unwrap();
    norm.drain(..s.len());

    let mut i = 0;
    loop {
        let Some(ch) = norm.get(i) else {
            // String finished
            break;
        };

        // U+00AD SOFT HYPHEN and U+034F COMBINING GRAPHEME JOINER create a blocking context
        // And break if ccc == 0
        // TODO: Not correct; check spec
        if matches!(ch, 173 | 847)
            || matches!(
                get_canonical_combining_class_u32(*ch),
                CanonicalCombiningClass::NotReordered
            )
        {
            break;
        }

        if let Some(ce) = lookup(&[s.as_slice(), &[*ch]].concat()) {
            // Restart from the start to handle discontiguous normalization
            res = ce;
            s.push(*ch);
            norm.remove(i);
            i = 0;
        } else {
            i += 1;
        }
    }

    res
}

pub fn get_collate_elements(s: &str) -> [Vec<u16>; 3] {
    let mut norm: Vec<u32> = s.nfd().map(|ch| ch as u32).collect();
    let mut weights = [vec![], vec![], vec![]];

    let mut push_ce = |arr: &[CollateElement]| {
        for ce in arr {
            for level in 0..=2 {
                if ce[level] != 0 {
                    weights[level].push(ce[level]);
                }
            }
        }
    };

    while !norm.is_empty() {
        push_ce(&find_next_longest_seq(&mut norm));
    }

    weights
}

fn build_sort_key(mut weights: [Vec<u16>; 3]) -> Vec<u16> {
    let mut sort_key = Vec::new();
    for i in 0..=2 {
        for w in &weights[i] {
            if *w != 0 {
                sort_key.push(*w);
            }
        }
        if i != 2 {
            sort_key.push(0);
        }
    }
    sort_key
}

pub fn sort_key(s: &str) -> Vec<u16> {
    build_sort_key(get_collate_elements(s))
}

pub fn collate(a: &str, b: &str) -> Ordering {
    // Fast path
    if a == b {
        return Ordering::Equal;
    }

    // // Primary
    // for

    sort_key(a).cmp(&sort_key(b))
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, io::BufRead, path::PathBuf};

    use crate::collate;

    #[test]
    fn check() {
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/non-ignorable.txt");
        let mut f = std::io::BufReader::new(std::fs::File::open(test_path).unwrap());

        let mut a = String::new();
        let mut b = String::new();
        let mut error_count = 0;
        let mut i = 0;
        f.read_line(&mut a).unwrap();
        a = a.trim().to_string();
        loop {
            if f.read_line(&mut b).unwrap() == 0 {
                break;
            } else {
                b = b.trim().to_string();
            }

            let a_cp: String = a
                .split(' ')
                .map(|s| {
                    let n = u32::from_str_radix(s, 16).unwrap();
                    char::from_u32(n).unwrap_or('\u{FFFD}')
                })
                .collect();
            let b_cp: String = b
                .split(' ')
                .map(|s| {
                    let n = u32::from_str_radix(s, 16).unwrap();
                    char::from_u32(n).unwrap_or('\u{FFFD}')
                })
                .collect();

            if matches!(collate(&a_cp, &b_cp), Ordering::Greater) {
                error_count += 1;
            }

            std::mem::swap(&mut a, &mut b);
            b.clear();
            i += 1;
        }
        assert_eq!(
            error_count,
            0,
            "{error_count}/{i} failed tests: {}%",
            100.0 * (1.0 - (error_count as f32 / i as f32))
        );
    }
}
