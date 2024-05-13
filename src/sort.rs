// Functions about the sorting

use crate::{Match, Chara, FightCond, groups::Tags, stats};
use std::io::{self, Write};
use std::str::FromStr;
use std::collections::HashSet;
use colored::Colorize;
use rand::distributions::{WeightedIndex, Distribution};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

// Performs one battle, creates a Match record and pushes it to the vec
// Returns a FightCond so the loop in main() knows what to do next
pub fn fight(records: &mut Vec<Match>, fire: &mut Chara, ice: &mut Chara, fire_id: usize, ice_id: usize)
-> FightCond {
    let mut choice: String = Default::default();
    loop {
        println!("-----------------------------");
        // println!("Battle #{}: {} ({:.0}) vs {} ({:.0})", records.len() + 1, fire.name.bold(), fire.rank.rate, ice.name.bold(), ice.rank.rate);
        println!("Battle #{}: {} vs {}", records.len() + 1, fire.name.bold(), ice.name.bold());
        print!("Pick [ 'h' for help ] >> ");
        let _ = io::stdout().flush();
        choice.clear();
        let _ = io::stdin().read_line(&mut choice);

        let mut game = Match {
            one: fire_id,
            two: ice_id,
            res: 1.0,
        };

        choice = choice.trim().to_string();
        if choice.ends_with('1') {
            // I like left
            println!("Chose - {}!", fire.name.blue());
        } else if choice.ends_with('2') {
            // I like right
            game.res = 0.0;
            println!("Chose - {}!", ice.name.blue());
        } else if choice.starts_with("end") {
            // End
            println!("Finishing rating period...");
            return FightCond::Last;
        } else if choice.ends_with('d') {
            // I dislike them both! (special value)
            game.res = 2.0;
            println!("Disliked both!");
        } else if choice.ends_with('l') {
            // Undo
            if records.len() == 0 {
                println!("This is the first battle!");
                continue;
            }
            println!("Going back...");
            return FightCond::Undo;
        } else if choice.ends_with('h') {
            // Help
            println!("1/2 to choose left/right");
            println!("<Enter> for draws");
            println!("d if you DISLIKE BOTH of them");
            println!("l to undo");
            println!("\"end\" to end this session");
            continue;
        } else {
            game.res = 0.5;
            println!("Chose - Draw!");
        }

        records.push(game);
        return FightCond::Next;
    }
}

// Picks two characters to fight, returns the indices within the pool
pub fn matchmake(rng: &mut ThreadRng, pool: &Vec<&mut Chara>, unranked_picks: &mut HashSet<usize>)
-> Vec<usize> {
    let pool_size = pool.len();
    let mut pair_id: Vec<usize> = Vec::with_capacity(2);
    // when a character hasn't had a match yet, we ensure one of them is included,
    // getting everyone off the start quickly
    let mut unranked_pool = pool.iter().enumerate()
        .filter(|(n, a)| {
            a.hist.wins + a.hist.draw + a.hist.loss == 0 && !unranked_picks.contains(n)
        })
        .map(|(n, _)| n).peekable();
    if unranked_pool.peek().is_some() {
        pair_id.push(unranked_pool.choose(rng).unwrap());
    } else {
        // otherwise just pick someone, favoring higher rated characters
        let dist = WeightedIndex::new(pool.iter().map(|a| a.rank.rate)).unwrap();
        pair_id.push(dist.sample(rng));
    }

    // pick the second character, favoring someone with similiar rating
    let mut scores: Vec<f64> = Vec::with_capacity(pool_size - 1);
    for th in pool.iter() {
        if th.name == pool[pair_id[0]].name {
            scores.push(0.0);                                       // don't pick the same person again
        } else {
            let diff = th.rank.rate - pool[pair_id[0]].rank.rate;
            let s = std::f64::consts::E.powf(diff.abs() / -135.0);  // exponential decay
            scores.push(s);
        }
    }
    let dist = WeightedIndex::new(&scores).unwrap();
    pair_id.push(dist.sample(rng));                                 // weighted random sampling
    pair_id
}

// Parses line for tags (series, stages) and flags (pc98, notgirl, and nameless)
// Consumes line and returns a vec of tags and array of bool flags
pub fn parse_filter(line: String)
-> (Vec<(Tags, bool)>, [bool; 3]) {
    let mut tags: Vec<(Tags, bool)> = Vec::new();
    // Default: no pc98, no non-girls, include nameless
    let mut flags: [bool; 3] = [false, false, true];
    let linev: Vec<&str> = line.split(" ").into_iter().collect();

    // remove unrecognised tokens
    let verify = |a: &&str| {
        *a == ""
        ||  a.contains("pc98")
        ||  a.contains("notgirl")
        ||  a.contains("namel")
        ||  if a.starts_with("-") {
                Tags::from_str(&a.trim()[1..]).is_ok()
            } else {
                Tags::from_str(a.trim()).is_ok()
            }
    };
    let (linev, invalid): (Vec<&str>, _) = linev.into_iter()
        .partition(verify);
    for word in invalid.iter() {
        println!("Unrecognised: {}", word);
    }

    // build the tags vector
    for token in linev.iter() {
        let is_excl = token.starts_with("-");
        let token_unsigned = if is_excl {
            &token[1..]
        } else {
            &token[..]
        };
        match Tags::from_str(token_unsigned) {
            Ok(t) => {
                println!("Filter: {} {}",
                    if is_excl {
                        "Excluding".red()
                    } else {
                        "Including".blue()
                    },
                    t.name().bold()
                );
                tags.push((t, !is_excl));
            },
            Err(_) => {
                match token {
                    t if t.contains("pc98")     => {
                        if t.starts_with("-") {
                            println!("Note: PC-98 duplicates are excluded by default.");
                        } else {
                            flags[0] = true;
                        }
                    },
                    t if t.contains("notgirl")  => {
                        if t.starts_with("-") {
                            println!("Note: Non-girls are excluded by default.");
                        } else {
                            flags[1] = true;
                        }
                    },
                    t if t.contains("namel")   => {
                        if t.starts_with("-") {
                            flags[2] = false;
                        } else {
                            println!("Note: Nameless characters are included by default.");
                        }
                    },
                    _ => { /* shouldn't happen? */ },
                }
            },
        }
    }
    (tags, flags)
}

// Takes a line of user filters and generates the pool of contestants, consumes line and
// returns vec of (references of) all qualified characters, and their respective indices
// in the original array.
pub fn bouncer(line: String, touhous: &mut Vec<Chara>)
-> (Vec<&mut Chara>, Vec<usize>) {
    // filter by tags
    let (tags, flags): (Vec<(Tags, bool)>, [bool; 3]) = parse_filter(line);
    let (mut filtered, mut indices): (Vec<&mut Chara>,_) = stats::filter_group_mut(tags, touhous);
    // filter by flags (pc98, nongirls, nameless)
    // flags[]: remove if 0
    let flag_filter = |a: &&mut Chara| {
        !(!flags[0] && a.is_pc98()
            || !flags[1] && a.is_not_girl()
            || !flags[2] && a.is_nameless())
    };
    // we want to filter both filtered() and indices() at once
    let to_remove: Vec<usize> = filtered.iter().enumerate()
        .filter(|(_, &ref th)| !flag_filter(&th) /* remove if false */)
        .map(|(id, _)| id)
        .collect();
    let mut offset = 0;
    for id in to_remove {
        filtered.remove(id - offset);
        indices.remove(id - offset);
        offset += 1; // as we remove the index is shifted
    }
    // final pool
    (filtered, indices)
}