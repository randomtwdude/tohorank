// Functions for the statistics feature

use crate::{Chara, Tags};
use colored::Colorize;

// Get the ranking of a character in the pool
pub fn rank_in_group(touhou: &Chara, pool: &Vec<&Chara>)
-> (usize, usize) {
    let mut rank = 1;
    let benchmark = touhou.rank.rate;
    for th in pool.iter().filter(|t| !t.dont_know()) {
        if th.rank.rate > benchmark {
            rank += 1;
        }
    }
    (rank, pool.iter().filter(|t| !t.dont_know()).count())
}

// Filter characters in pool by tags
// Every tag is either inclusive (true) or exclusive (false)
pub fn filter_group<'a>(tags: Vec<(Tags, bool)>, pool: &'a Vec<Chara>)
-> Vec<&'a Chara> {
    let mut filtered: Vec<&Chara> = Vec::new();
    // split the two types of tags
    let (series_t, stages_t): (Vec<_>, Vec<_>) = tags.into_iter().
        partition(|a| a.0.is_series_tag());

    // first round: series tags
    for th in pool.iter() {
        // bit long innit
        let no_specified_incl_tags = series_t.iter().filter(|a| a.1).collect::<Vec<_>>().is_empty();
        let has_any_incl_tags = series_t.iter().filter(|a| a.1).any(|tag| th.has_tag(&tag.0));
        let has_no_excl_tags = series_t.iter().filter(|a| !a.1).all(|tag| !th.has_tag(&tag.0));

        if series_t.is_empty()
           || (no_specified_incl_tags || has_any_incl_tags) && has_no_excl_tags
        {
            filtered.push(th);
        }
    }
    // second round: stages tags
    let stage_filter = |th: &&Chara| {
        let no_specified_incl_tags = stages_t.iter().filter(|a| a.1).collect::<Vec<_>>().is_empty();
        let has_any_incl_tags = stages_t.iter().filter(|a| a.1).any(|tag| th.has_tag(&tag.0));
        let has_no_excl_tags = stages_t.iter().filter(|a| !a.1).all(|tag| !th.has_tag(&tag.0));

        stages_t.is_empty()
           || (no_specified_incl_tags || has_any_incl_tags) && has_no_excl_tags
    };
    filtered.retain(stage_filter);
    // final result
    filtered
}

// Same as above but returns mutable references, for sorting
// also returns a mapping of filtered indices to unfiltered indices
pub fn filter_group_mut<'a>(tags: Vec<(Tags, bool)>, pool: &'a mut Vec<Chara>)
-> (Vec<&'a mut Chara>, Vec<usize>) {
    let mut filtered: Vec<&mut Chara> = Vec::new();
    let mut indices: Vec<usize> = Vec::with_capacity(filtered.len());
    let (series_t, stages_t): (Vec<_>, Vec<_>) = tags.into_iter().
        partition(|a| a.0.is_series_tag());
    for (id, th) in pool.iter_mut().enumerate() {
        let no_specified_incl_tags = series_t.iter().filter(|a| a.1).collect::<Vec<_>>().is_empty();
        let has_any_incl_tags = series_t.iter().filter(|a| a.1).any(|tag| th.has_tag(&tag.0));
        let has_no_excl_tags = series_t.iter().filter(|a| !a.1).all(|tag| !th.has_tag(&tag.0));

        if series_t.is_empty()
           || (no_specified_incl_tags || has_any_incl_tags) && has_no_excl_tags
        {
            filtered.push(th);
            indices.push(id);
        }
    }
    let stage_filter = |th: &&mut Chara| {
        let no_specified_incl_tags = stages_t.iter().filter(|a| a.1).collect::<Vec<_>>().is_empty();
        let has_any_incl_tags = stages_t.iter().filter(|a| a.1).any(|tag| th.has_tag(&tag.0));
        let has_no_excl_tags = stages_t.iter().filter(|a| !a.1).all(|tag| !th.has_tag(&tag.0));

        stages_t.is_empty()
           || (no_specified_incl_tags || has_any_incl_tags) && has_no_excl_tags
    };
    // we want to filter both filtered() and indices() at once, so retain() doesn't cut it
    // or maybe it does and I'm just bad.
    let to_remove: Vec<usize> = filtered.iter().enumerate()
        .filter(|(_, &ref th)| !stage_filter(&th) /* remove if false */)
        .map(|(id, _)| id)
        .collect();
    let mut offset = 0;
    for id in to_remove {
        filtered.remove(id - offset);
        indices.remove(id - offset);
        offset += 1; // as we remove the index is shifted
    }
    (filtered, indices)
}

// Get slice of ranking around the character in a group
pub fn rank_slice_by_chara<'a>(chara: &'a Chara, pool: &'a Vec<&'a Chara>)
-> Vec<&'a Chara> {
    let mut poolc = pool.clone();
    poolc.retain(|a| !a.dont_know());
    poolc.sort_by(|a, b| b.rank.rate.partial_cmp(&a.rank.rate).unwrap());
    match poolc.len() {
        ..=3 => poolc,
        4.. => {
            for (n, th) in poolc.iter().enumerate() {
                if chara.name == th.name {
                    if n == 0 {
                        return vec![poolc[0], poolc[1], poolc[2]];          // top
                    } else if n == poolc.len() - 1 {
                        return vec![poolc[n - 2], poolc[n - 1], poolc[n]];  // bottom
                    } else {
                        return vec![poolc[n - 1], poolc[n], poolc[n + 1]];  // middle
                    }
                }
            }
            vec![]
        }
    }
}

// Prints group rankings in stats screen, takes one tag, it only displays one
pub fn print_rank_in_group(chara: &Chara, tag: Vec<(Tags, bool)>, pool: &Vec<Chara>) {
    // borrow checker complaining? just clone() !
    let group = filter_group(tag.clone(), pool);
    let rank = rank_in_group(chara, &group);
    // title
    println!("\n  - {:<42}{:>8}",
        if tag.len() > 0 {
            if tag[0].0.exname() != "" {
                format!("in TH{}", tag[0].0.exname())
            } else {
                format!("in {}", tag[0].0.name())
            }
        } else {
            "Overall".to_string()
        },
        format!("#{}/{}", rank.0, rank.1)
    );
    // more title
    if tag.len() > 0 && tag[0].0.exname() != "" {
        println!("    {:^50}", tag[0].0.name().bold());
    }
    println!("    {:-<50}", "");
    let rank_list = rank_slice_by_chara(chara, &group);
    for th in rank_list.iter() {
        let rank = rank_in_group(th, &group).0;
        let entry = format!("    {:<4} {:<34} {:>10}",
            format!("{}.", rank),
            if th.name == chara.name {
                th.name.bold()
            } else {
                th.name.normal()
            },
            format!("{} ± {:.0}",
                format!("{:.0}", th.rank.rate).bold(),
                th.rank.devi * 1.96
            )
        );
        match rank {
            1 => { println!("{}", entry.truecolor(245, 212, 95)); },
            2 => { println!("{}", entry.truecolor(180, 245, 212)); },
            3 => { println!("{}", entry.truecolor(240, 140, 95)); },
            _ => { println!("{}", entry); },
        }
    }
}