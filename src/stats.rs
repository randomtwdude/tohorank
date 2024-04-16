// Functions for the statistics feature
use crate::{Chara, Tags};
use colored::Colorize;

// Get the ranking in a group
pub fn rank_in_group(touhou: &Chara, pool: &Vec<&Chara>) -> (usize, usize) {
    let mut rank = 1;
    let benchmark = touhou.rank.rate;
    for th in pool.iter().filter(|t| !t.dont_know()) {
        if th.rank.rate > benchmark {
            rank += 1;
        }
    }
    // ???
    (rank, pool.iter().filter(|t| !t.dont_know()).collect::<Vec<_>>().len())
}

// Get all characters with some tags, if tags is empty, returns all
pub fn filter_group<'a>(tags: &'a Vec<Tags>, pool: &'a Vec<Chara>) -> Vec<&'a Chara> {
    let mut filtered: Vec<&Chara> = Vec::new();
    for th in pool.iter() {
        if tags.iter().all(|tag| th.has_tag(tag)) {
            filtered.push(th);
        }
    }
    filtered
}
pub fn filter_group_mut<'a>(tags: &'a Vec<Tags>, pool: &'a mut Vec<Chara>) -> Vec<&'a mut Chara> {
    let mut filtered: Vec<&mut Chara> = Vec::new();
    for th in pool.iter_mut() {
        if tags.iter().all(|tag| th.has_tag(tag)) {
            filtered.push(th);
        }
    }
    filtered
}

// Get slice of ranking around character in a group
pub fn rank_slice_by_chara<'a>(chara: &'a Chara, pool: &'a Vec<&'a Chara>) -> Vec<&'a Chara> {
    let mut poolc = pool.clone();
    poolc.retain(|a| !a.dont_know());
    poolc.sort_by(|a, b| b.rank.rate.partial_cmp(&a.rank.rate).unwrap());
    match poolc.len() {
        ..=3 => poolc,
        4.. => {
            for (n, th) in poolc.iter().enumerate() {
                if chara.name == th.name {
                    if n == 0 {
                        // top
                        return vec![poolc[0], poolc[1], poolc[2]];
                    } else if n == poolc.len() - 1 {
                        // bottom
                        return vec![poolc[n - 2], poolc[n - 1], poolc[n]];
                    } else {
                        // middle
                        return vec![poolc[n - 1], poolc[n], poolc[n + 1]];
                    }
                }
            }
            vec![]
        }
    }
}

// Print rankings in stats, takes one tag
pub fn print_rank_in_group(chara: &Chara, tag: &Vec<Tags>, pool: &Vec<Chara>) {
    // borrow checker complaining? just clone() !
    let group = filter_group(tag, pool);
    let rank = rank_in_group(chara, &group);
    println!("\n    - {}: #{}/{}",
        if tag.len() > 0 {
            format!("in {}", tag.get(0).unwrap().name().bold())
        } else {
            "Overall".to_string()
        },
        rank.0,
        rank.1
    );
    println!("    {:-<42}", "");
    let rank_list = rank_slice_by_chara(chara, &group);
    for th in rank_list.iter() {
        println!("    {0: <4} {1: <26} {2} ± {3:.0}",
            format!("{}.", rank_in_group(th, &group).0),
            th.name,
            format!("{:.0}", th.rank.rate).bold(),
            th.rank.devi * 1.96
        );
    }
}