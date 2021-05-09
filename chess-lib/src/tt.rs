use crate::types::ZobristHash;

pub struct TranspositionTable<T: Clone + Copy> {
    shift: u32,
    table: Vec<Option<TTNode<T>>>,
    collision_policy: CollisionPolicy<T>,
    stats: TTStats,
}

#[derive(Clone, Copy)]
pub struct TTNode<T: Clone + Copy> {
    zh: ZobristHash,
    data: T,
}

impl <T: Clone + Copy> TTNode<T> {
    pub fn new(zh: ZobristHash, data: T) -> TTNode<T> {
        TTNode{zh, data}
    }
}

pub struct TTStats {
    hits: u64,
    collisions: u64,
    total: u64,
    size: usize,
    filled: usize,
}

impl TTStats {
    pub fn hit_rate(&self) -> f64 {
        (self.hits as f64) / (self.total as f64)
    }

    pub fn collision_rate(&self) -> f64 {
        (self.collisions as f64) / (self.total as f64)
    }

    pub fn fill_rate(&self) -> f64 {
        (self.filled as f64) / (self.size as f64)
    }
}

pub enum PolicyResult {
    Keep,
    Replace,
}

pub type CollisionPolicy<T> = fn (T, T) -> PolicyResult;

pub fn always_replace<T>(_: T, _: T) -> PolicyResult {
    PolicyResult::Replace
}

pub fn never_replace<T>(_: T, _: T) -> PolicyResult {
    PolicyResult::Keep
}

impl <T: Clone + Copy> TranspositionTable<T> {
    pub fn new(max_bytes: usize, collision_policy: CollisionPolicy<T>) -> TranspositionTable<T> {
        let entry_size = std::mem::size_of::<TTNode<T>>();
        let num_entries = (max_bytes / entry_size).next_power_of_two() >> 1;
        TranspositionTable{
            shift: 64 - num_entries.trailing_zeros(),
            table: vec![None; num_entries],
            collision_policy,
            stats: TTStats{
                hits: 0,
                collisions: 0,
                total: 0,
                size: num_entries,
                filled: 0,
            },
        }
    }

    pub fn stats(&self) -> &TTStats {
        &self.stats
    }

    pub fn get(&mut self, zh: ZobristHash) -> Option<T> {
        self.stats.total += 1;
        self.table[self.index(zh)].and_then(|nd| {
            if nd.zh == zh {
                self.stats.hits += 1;
                Some(nd.data)
            } else {
                self.stats.collisions += 1;
                None
            }
        })
    }

    pub fn get_or_compute<F: FnOnce() -> T>(&mut self, zh: ZobristHash, compute: F) -> T {
        self.stats.total += 1;
        let ix = self.index(zh);
        match self.table[ix].take() {
            Some(prev) => {
                println!("{:?} =? {:?}  {}", prev.zh, zh, prev.zh == zh);
                if prev.zh == zh {
                    self.stats.hits += 1;
                    let data = prev.data;
                    self.table[ix].replace(prev);
                    data
                } else {
                    self.stats.collisions += 1;
                    let data = compute();
                    match (self.collision_policy)(prev.data, data) {
                        PolicyResult::Keep => self.table[ix].replace(prev),
                        PolicyResult::Replace => self.table[ix].replace(TTNode::new(zh, data)),
                    };
                    data
                }
            },
            None => {
                self.stats.filled += 1;
                let data = compute();
                self.table[ix].replace(TTNode::new(zh, data));
                data
            },
        }
    }

    pub fn insert(&mut self, zh: ZobristHash, data: T) {
        let ix = self.index(zh);
        match self.table[ix].take() {
            Some(prev) => {
                if prev.zh == zh {
                    self.table[ix].replace(prev)
                } else {
                    match (self.collision_policy)(prev.data, data) {
                        PolicyResult::Keep => self.table[ix].replace(prev),
                        PolicyResult::Replace => self.table[ix].replace(TTNode::new(zh, data)),
                    }
                }
            },
            None => {
                self.stats.filled += 1;
                self.table[ix].replace(TTNode::new(zh, data))
            },
        };
    }

    fn index(&self, zh: ZobristHash) -> usize {
        (zh.0 >> self.shift) as usize
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    use crate::tt::*;
    use crate::types::ZobristHash;

    #[test]
    fn test_size() {
        let tt = TranspositionTable::<ZobristHash>::new(1 << 16, always_replace);
        assert_eq!(tt.table.len() * std::mem::size_of::<TTNode<ZobristHash>>(), 32768);
    }

    #[test]
    fn test_get_and_retrieve() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let mut tt = TranspositionTable::<u64>::new(1 << 16, always_replace);

        // With always_replace policy we should be able to loop get/retrieve forever.
        for _ in 0..100_000 {
            let zh = ZobristHash(rng.gen());
            tt.insert(zh, zh.0);
            assert_eq!(tt.get(zh), Some(zh.0));
        }
    }

    #[test]
    fn test_no_replace() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let mut tt = TranspositionTable::<u64>::new(1 << 8, never_replace);

        let zh = ZobristHash(rng.gen());
        let collision = generate_collision(&tt, zh);

        tt.insert(zh, 42);

        // Inserting with same zh won't replace.
        tt.insert(zh, 0);
        assert_eq!(tt.get(zh), Some(42));

        // Inserting with colliding zh also won't replace due to policy.
        tt.insert(collision, 0);
        assert_eq!(tt.get(zh), Some(42));

    }

    #[test]
    fn test_get_or_compute_no_replace() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let mut tt = TranspositionTable::<u64>::new(1 << 8, never_replace);

        let zh = ZobristHash(rng.gen());
        let collision = generate_collision(&tt, zh);

        assert_eq!(tt.get_or_compute(zh, || 42), 42);

        // Never replace.
        assert_eq!(tt.get_or_compute(zh, || panic!("Should never be called")), 42);

        // In case of index collision, the new result will be computed but not stored!
        assert_eq!(tt.get_or_compute(collision, || 0), 0);
        assert_eq!(tt.get(zh), Some(42));
    }

    #[test]
    fn test_get_or_compute_replace() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let mut tt = TranspositionTable::<u64>::new(1 << 8, always_replace);

        let zh = ZobristHash(rng.gen());
        let collision = generate_collision(&tt, zh);

        assert_eq!(tt.get_or_compute(zh, || 42), 42);
       
        // On hash collision we don't recompute.
        assert_eq!(tt.get_or_compute(zh, || panic!("Should never be called")), 42);

        // On index collision we do.
        assert_eq!(tt.get_or_compute(collision, || 0), 0);

        // The original hash is now not present.
        assert_eq!(tt.get(zh), None);
    }

    fn generate_collision<T: Copy>(tt: &TranspositionTable<T>, zh: ZobristHash) -> ZobristHash {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        loop {
            let collision = ZobristHash(rng.gen());
            if tt.index(collision) == tt.index(zh) && collision != zh {
                return collision;
            }
        }
    }
}
