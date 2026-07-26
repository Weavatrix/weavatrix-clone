use std::collections::{HashSet, VecDeque};

const BASE: u64 = 1_099_511_628_211;

pub(crate) fn winnow(tokens: &[u32], k: usize, window: usize) -> Vec<u64> {
    if tokens.len() < k + window - 1 {
        return Vec::new();
    }
    let mut deque = VecDeque::<(usize, u64)>::with_capacity(window);
    let hashes = rolling_hashes(tokens, k);
    let mut selected = HashSet::with_capacity(hashes.len() / window + 1);
    for (index, hash) in hashes.enumerate() {
        while deque.back().is_some_and(|(_, value)| *value >= hash) {
            deque.pop_back();
        }
        deque.push_back((index, hash));
        while deque
            .front()
            .is_some_and(|(position, _)| position + window <= index)
        {
            deque.pop_front();
        }
        if index + 1 >= window {
            selected.insert(deque.front().expect("winnowing window is nonempty").1);
        }
    }
    let mut fingerprints = selected.into_iter().collect::<Vec<_>>();
    fingerprints.sort_unstable();
    fingerprints
}

pub(crate) fn rolling_hashes<T>(tokens: &[T], k: usize) -> RollingHashes<'_, T>
where
    T: Copy + Into<u64>,
{
    let valid = k != 0 && tokens.len() >= k;
    let mut factor = 1_u64;
    for _ in 1..k {
        factor = factor.wrapping_mul(BASE);
    }
    let mut hash = 0_u64;
    for token in tokens.get(..k).unwrap_or_default() {
        hash = hash
            .wrapping_mul(BASE)
            .wrapping_add((*token).into().wrapping_add(1));
    }
    RollingHashes {
        tokens,
        k,
        factor,
        hash,
        next: 0,
        count: usize::from(valid) * (tokens.len().saturating_sub(k) + 1),
    }
}

pub(crate) struct RollingHashes<'a, T> {
    tokens: &'a [T],
    k: usize,
    factor: u64,
    hash: u64,
    next: usize,
    count: usize,
}

impl<T> Iterator for RollingHashes<'_, T>
where
    T: Copy + Into<u64>,
{
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.count {
            return None;
        }
        if self.next != 0 {
            let outgoing = self.tokens[self.next - 1].into().wrapping_add(1);
            self.hash = self.hash.wrapping_sub(outgoing.wrapping_mul(self.factor));
            self.hash = self
                .hash
                .wrapping_mul(BASE)
                .wrapping_add(self.tokens[self.next + self.k - 1].into().wrapping_add(1));
        }
        self.next += 1;
        Some(self.hash)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count.saturating_sub(self.next);
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for RollingHashes<'_, T> where T: Copy + Into<u64> {}

#[cfg(test)]
mod tests {
    use super::winnow;

    #[test]
    fn winnowing_is_stable_and_change_sensitive() {
        let input = (0..100).collect::<Vec<_>>();
        assert_eq!(winnow(&input, 8, 4), winnow(&input, 8, 4));
        let mut changed = input;
        changed[50] = 999;
        assert_ne!(
            winnow(&changed, 8, 4),
            winnow(&(0..100).collect::<Vec<_>>(), 8, 4)
        );
    }
}
