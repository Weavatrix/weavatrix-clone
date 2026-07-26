use crate::fast_hash::{FastMap, SeededBuildHasher};
use crate::{CloneConfig, CloneError, Result, Similarity};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Candidate {
    pub left: usize,
    pub right: usize,
    pub shared: usize,
    pub jaccard: Similarity,
    pub containment: Similarity,
}

#[derive(Debug, Default)]
pub(crate) struct CandidateIndex {
    pub candidates: Vec<Candidate>,
    pub suppressed_buckets: usize,
}

pub(crate) fn candidates(fingerprints: &[Vec<u64>], config: CloneConfig) -> Result<CandidateIndex> {
    let mut inverted = FastMap::<u64, Vec<u32>>::with_hasher(SeededBuildHasher::random());
    for (fragment, values) in fingerprints.iter().enumerate() {
        let fragment = u32::try_from(fragment).map_err(|_| CloneError::CapacityExceeded {
            resource: "fragment index",
            limit: u32::MAX as usize,
        })?;
        for value in values {
            inverted.entry(*value).or_default().push(fragment);
        }
    }

    let mut effective = vec![0_usize; fingerprints.len()];
    let mut shared = FastMap::<u64, u32>::with_hasher(SeededBuildHasher::random());
    let mut suppressed_buckets = 0;
    for bucket in inverted.values() {
        if bucket.len() > config.max_bucket_size {
            suppressed_buckets += 1;
            continue;
        }
        for index in bucket {
            effective[*index as usize] += 1;
        }
        for left in 0..bucket.len() {
            for right in left + 1..bucket.len() {
                let key = pair_key(bucket[left], bucket[right]);
                if !shared.contains_key(&key) && shared.len() >= config.max_candidates {
                    return Err(CloneError::CapacityExceeded {
                        resource: "candidate pairs",
                        limit: config.max_candidates,
                    });
                }
                *shared.entry(key).or_default() += 1;
            }
        }
    }

    let mut result = Vec::with_capacity(shared.len());
    for (key, shared_count) in shared {
        let (left, right) = unpack_pair(key);
        let count = shared_count as usize;
        if count < config.min_shared_fingerprints {
            continue;
        }
        let union = effective[left]
            .saturating_add(effective[right])
            .saturating_sub(count);
        let smaller = effective[left].min(effective[right]);
        let jaccard = Similarity::from_ratio(count, union);
        let containment = Similarity::from_ratio(count, smaller);
        if jaccard.max(containment) < config.candidate_similarity {
            continue;
        }
        result.push(Candidate {
            left,
            right,
            shared: count,
            jaccard,
            containment,
        });
    }
    result.sort_unstable_by_key(|candidate| (candidate.left, candidate.right));
    Ok(CandidateIndex {
        candidates: result,
        suppressed_buckets,
    })
}

fn pair_key(left: u32, right: u32) -> u64 {
    u64::from(left) << 32 | u64::from(right)
}

fn unpack_pair(key: u64) -> (usize, usize) {
    let left = u32::try_from(key >> 32).expect("packed pair left index fits u32");
    let right = u32::try_from(key & u64::from(u32::MAX)).expect("packed pair right index fits u32");
    (
        usize::try_from(left).expect("u32 fragment index fits usize"),
        usize::try_from(right).expect("u32 fragment index fits usize"),
    )
}
