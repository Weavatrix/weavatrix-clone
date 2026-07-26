#[cfg(feature = "scan")]
use std::collections::HashSet;
use std::collections::{HashMap, hash_map::RandomState};
use std::hash::{BuildHasher, Hasher};

pub(crate) type FastMap<K, V> = HashMap<K, V, SeededBuildHasher>;
#[cfg(feature = "scan")]
pub(crate) type FastSet<K> = HashSet<K, SeededBuildHasher>;

#[derive(Clone)]
pub(crate) struct SeededBuildHasher {
    seed: u64,
}

impl SeededBuildHasher {
    pub fn random() -> Self {
        let state = RandomState::new();
        let mut hasher = state.build_hasher();
        hasher.write_u64(0x7765_6176_6174_7269);
        Self {
            seed: hasher.finish(),
        }
    }
}

impl BuildHasher for SeededBuildHasher {
    type Hasher = MixedHasher;

    fn build_hasher(&self) -> Self::Hasher {
        MixedHasher(self.seed)
    }
}

pub(crate) struct MixedHasher(u64);

#[cfg(feature = "scan")]
pub(crate) fn stable_bytes(bytes: &[u8]) -> u64 {
    let mut value = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        value = (value ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    mix(value)
}

impl Hasher for MixedHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut value = self.0 ^ 0xcbf2_9ce4_8422_2325;
        for byte in bytes {
            value = (value ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 = mix(value);
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = mix(value ^ self.0);
    }
}

const fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{FastMap, SeededBuildHasher};

    #[test]
    fn map_handles_distinct_and_repeated_integer_keys() {
        let mut map = FastMap::with_hasher(SeededBuildHasher::random());
        map.insert(1_u64, "one");
        map.insert(2, "two");
        map.insert(1, "updated");
        assert_eq!(map.get(&1), Some(&"updated"));
        assert_eq!(map.get(&2), Some(&"two"));
    }
}
