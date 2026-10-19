//! Canonical Unicode 17 comparison without normalizing stored text or allocating.
//! Short nonstarter runs use fixed scratch. Longer runs are rescanned once per
//! occupied combining class (at most 255 passes), preserving equal-class order.
use core::{cmp::Ordering, iter::Peekable, str::Chars};

fn combining(scalar: u32) -> u8 {
    let table = crate::string_normalization_data::COMBINING;
    table
        .binary_search_by_key(&scalar, |entry| entry.0)
        .map_or(0, |index| table[index].1)
}

#[derive(Clone)]
struct Decomposed<'a> {
    input: Chars<'a>,
    pending: [u32; 4],
    position: usize,
    length: usize,
}
impl Iterator for Decomposed<'_> {
    type Item = (u32, u8);
    fn next(&mut self) -> Option<Self::Item> {
        if self.position == self.length {
            let scalar = self.input.next()? as u32;
            self.position = 0;
            self.length = 0;
            if (0xAC00..0xAC00 + 11172).contains(&scalar) {
                let syllable = scalar - 0xAC00;
                self.pending[0] = 0x1100 + syllable / 588;
                self.pending[1] = 0x1161 + (syllable % 588) / 28;
                self.pending[2] = 0x11A7 + syllable % 28;
                self.length = if syllable % 28 == 0 { 2 } else { 3 };
            } else {
                let table = crate::string_normalization_data::DECOMPOSITION;
                if let Ok(index) = table.binary_search_by_key(&scalar, |entry| entry.0) {
                    let expansion = table[index].1;
                    self.pending[..expansion.len()].copy_from_slice(expansion);
                    self.length = expansion.len();
                } else {
                    return Some((scalar, combining(scalar)));
                }
            }
        }
        let scalar = self.pending[self.position];
        self.position += 1;
        Some((scalar, combining(scalar)))
    }
}

type Cursor<'a> = Peekable<Decomposed<'a>>;
struct LongRun<'a> {
    start: Cursor<'a>,
    scan: Cursor<'a>,
    count: usize,
    remaining: usize,
    classes: [u64; 4],
    class: u8,
}
impl LongRun<'_> {
    fn next(&mut self) -> Option<u32> {
        loop {
            while self.remaining != 0 {
                self.remaining -= 1;
                let (scalar, class) = self.scan.next().expect("measured nonstarter run");
                if class == self.class {
                    return Some(scalar);
                }
            }
            let (word, bits) = self
                .classes
                .iter_mut()
                .enumerate()
                .find(|(_, bits)| **bits != 0)?;
            let bit = bits.trailing_zeros();
            *bits &= *bits - 1;
            self.class = (word * 64 + bit as usize) as u8;
            self.scan = self.start.clone();
            self.remaining = self.count;
        }
    }
}

/// Streams NFD scalars. Scratch space is independent of input length.
pub(crate) struct Canonical<'a> {
    input: Cursor<'a>,
    short: [(u32, u8); 32],
    position: usize,
    length: usize,
    long: Option<LongRun<'a>>,
}
impl<'a> Canonical<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            input: Decomposed {
                input: text.chars(),
                pending: [0; 4],
                position: 0,
                length: 0,
            }
            .peekable(),
            short: [(0, 0); 32],
            position: 0,
            length: 0,
            long: None,
        }
    }
}
impl Iterator for Canonical<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        loop {
            if self.position < self.length {
                let scalar = self.short[self.position].0;
                self.position += 1;
                return Some(scalar);
            }
            if let Some(long) = &mut self.long {
                if let Some(scalar) = long.next() {
                    return Some(scalar);
                }
                self.long = None;
            }
            let start = self.input.clone();
            let (scalar, class) = self.input.next()?;
            if class == 0 {
                return Some(scalar);
            }
            self.short[0] = (scalar, class);
            let mut count = 1;
            let mut classes = [0u64; 4];
            classes[usize::from(class / 64)] |= 1u64 << (class % 64);
            while self.input.peek().is_some_and(|entry| entry.1 != 0) {
                let entry = self.input.next().expect("peeked nonstarter");
                if count < self.short.len() {
                    self.short[count] = entry;
                }
                count += 1;
                classes[usize::from(entry.1 / 64)] |= 1u64 << (entry.1 % 64);
            }
            self.position = 0;
            if count <= self.short.len() {
                self.length = count;
                // Stable insertion sort, bounded to 32 entries; equal classes retain order.
                for index in 1..count {
                    let mut position = index;
                    while position > 0 && self.short[position - 1].1 > self.short[position].1 {
                        self.short.swap(position - 1, position);
                        position -= 1;
                    }
                }
            } else {
                self.length = 0;
                self.long = Some(LongRun {
                    scan: start.clone(),
                    start,
                    count,
                    remaining: 0,
                    classes,
                    class: 0,
                });
            }
        }
    }
}

pub(crate) fn compare(left: &str, right: &str) -> Ordering {
    if left == right {
        Ordering::Equal
    } else if left.is_ascii() && right.is_ascii() {
        left.cmp(right)
    } else {
        Canonical::new(left).cmp(Canonical::new(right))
    }
}

pub(crate) fn hash(text: &str) -> i32 {
    let mut hash = 2166136261u32;
    let mut add = |byte: u8| {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    };
    if text.is_ascii() {
        text.bytes().for_each(&mut add);
    } else {
        let mut bytes = [0; 4];
        for scalar in Canonical::new(text) {
            let scalar = char::from_u32(scalar).expect("generated Unicode scalar");
            scalar.encode_utf8(&mut bytes).bytes().for_each(&mut add);
        }
    }
    hash as i32
}
