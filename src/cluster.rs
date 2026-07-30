use crate::model::{CloneFamily, CloneLocation, ClonePair};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct PairMembership {
    pub left: usize,
    pub right: usize,
    pub id: String,
}

pub(crate) fn families(locations: &[CloneLocation], pairs: &[PairMembership]) -> Vec<CloneFamily> {
    let mut parents = (0..locations.len()).collect::<Vec<_>>();
    for pair in pairs {
        union(&mut parents, pair.left, pair.right);
    }
    let mut grouped = HashMap::<usize, (Vec<usize>, Vec<String>)>::new();
    for pair in pairs {
        let root = find(&mut parents, pair.left);
        let group = grouped.entry(root).or_default();
        group.0.push(pair.left);
        group.0.push(pair.right);
        group.1.push(pair.id.clone());
    }
    let mut result = grouped
        .into_values()
        .map(|(mut indexes, mut pair_ids)| {
            indexes.sort_unstable();
            indexes.dedup();
            pair_ids.sort();
            pair_ids.dedup();
            let mut members = indexes
                .into_iter()
                .map(|index| locations[index].clone())
                .collect::<Vec<_>>();
            members.sort();
            let id = stable_id(
                "family",
                members
                    .iter()
                    .map(|member| member.fragment_id.as_str())
                    .chain(pair_ids.iter().map(String::as_str)),
            );
            CloneFamily {
                id,
                members,
                pair_ids,
            }
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| left.id.cmp(&right.id));
    result
}

pub(crate) fn families_for_pairs(pairs: &[ClonePair]) -> Vec<CloneFamily> {
    let mut locations = Vec::<CloneLocation>::new();
    let mut indexes = HashMap::<&CloneLocation, usize>::new();
    let mut memberships = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let left = location_index(&mut locations, &mut indexes, &pair.left);
        let right = location_index(&mut locations, &mut indexes, &pair.right);
        memberships.push(PairMembership {
            left,
            right,
            id: pair.id.clone(),
        });
    }
    families(&locations, &memberships)
}

fn location_index<'a>(
    locations: &mut Vec<CloneLocation>,
    indexes: &mut HashMap<&'a CloneLocation, usize>,
    location: &'a CloneLocation,
) -> usize {
    if let Some(index) = indexes.get(location) {
        return *index;
    }
    let index = locations.len();
    locations.push(location.clone());
    indexes.insert(location, index);
    index
}

pub(crate) fn pair_id(left: &CloneLocation, right: &CloneLocation) -> String {
    stable_id(
        "pair",
        [
            left.fragment_id.as_str(),
            right.fragment_id.as_str(),
            left.path.as_str(),
            right.path.as_str(),
        ],
    )
}

fn find(parents: &mut [usize], value: usize) -> usize {
    let mut root = value;
    while parents[root] != root {
        root = parents[root];
    }
    let mut current = value;
    while parents[current] != current {
        let next = parents[current];
        parents[current] = root;
        current = next;
    }
    root
}

fn union(parents: &mut [usize], left: usize, right: usize) {
    let left_root = find(parents, left);
    let right_root = find(parents, right);
    if left_root != right_root {
        parents[right_root] = left_root;
    }
}

fn stable_id<'a>(kind: &str, parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut first = 0xcbf2_9ce4_8422_2325_u64;
    let mut second = 0x9e37_79b9_7f4a_7c15_u64;
    for byte in kind
        .bytes()
        .chain(parts.into_iter().flat_map(|part| part.bytes().chain([0])))
    {
        first = (first ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        second ^= u64::from(byte).wrapping_add(first.rotate_left(17));
        second = second.wrapping_mul(0x9e37_79b1_85eb_ca87);
    }
    format!("wvxclone:v1:{kind}:{first:016x}{second:016x}")
}
