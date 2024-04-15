// Functions for the statistics feature
use crate::{Chara, Tags};

// Get the ranking in a group
pub fn rank_in_group(touhou: &Chara, pool: &Vec<Chara>) -> (usize, usize) {
    let mut rank = 1;
    let benchmark = touhou.rank.rate;
    for th in pool.iter() {
        if th.rank.rate > benchmark {
            rank += 1;
        }
    }
    (rank, pool.len())
}

// Get all characters with some tags, if tags is empty, returns all
pub fn filter_group(tags: Vec<Tags>, pool: &Vec<Chara>) -> Vec<&Chara> {
    let mut filtered: Vec<&Chara> = Vec::new();
    for th in pool.iter() {
        if tags.iter().all(|tag| th.has_tag(tag)) {
            filtered.push(&th);
        }
    }
    filtered
}