//! Simple BitSet implementation backed by a Vec<u64>.
//! Optimized for "no bloat" philosophy - minimal allocations, direct bitwise ops.

#[derive(Debug, Clone, Default)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    /// Create a new BitSet capable of holding at least `capacity` bits.
    pub fn with_capacity(capacity: usize) -> Self {
        let num_words = capacity.div_ceil(64);
        Self {
            words: vec![0; num_words],
        }
    }

    /// Set the bit at `index` to true.
    /// Resizes automatically if index is out of bounds.
    pub fn set(&mut self, index: usize) {
        let (word_idx, bit_idx) = (index / 64, index % 64);
        if word_idx >= self.words.len() {
            self.words.resize(word_idx + 1, 0);
        }
        self.words[word_idx] |= 1 << bit_idx;
    }

    /// Check if the bit at `index` is set.
    pub fn contains(&self, index: usize) -> bool {
        let (word_idx, bit_idx) = (index / 64, index % 64);
        if word_idx >= self.words.len() {
            return false;
        }
        (self.words[word_idx] & (1 << bit_idx)) != 0
    }

    /// Returns true if this set shares any set bits with `other`.
    pub fn intersects(&self, other: &Self) -> bool {
        let len = std::cmp::min(self.words.len(), other.words.len());
        for i in 0..len {
            if (self.words[i] & other.words[i]) != 0 {
                return true;
            }
        }
        false
    }

    /// Returns iterator over indices of set bits
    pub fn ones(&self) -> OnesIter {
        OnesIter {
            bitset: self,
            word_idx: 0,
            current_word: if self.words.is_empty() {
                0
            } else {
                self.words[0]
            },
        }
    }
}

pub struct OnesIter<'a> {
    bitset: &'a BitSet,
    word_idx: usize,
    current_word: u64,
}

impl<'a> Iterator for OnesIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_word != 0 {
                let trailing = self.current_word.trailing_zeros();
                self.current_word &= !(1 << trailing); // Clear the bit we just found
                return Some(self.word_idx * 64 + trailing as usize);
            }

            self.word_idx += 1;
            if self.word_idx >= self.bitset.words.len() {
                return None;
            }
            self.current_word = self.bitset.words[self.word_idx];
        }
    }
}
