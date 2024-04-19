// Functions about the sorting

use crate::{Match, Chara, FightCond, groups::Tags, stats};
use std::io::{self, Write};
use std::str::FromStr;
use std::collections::HashSet;
use colored::Colorize;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

// One battle, may add an entry to *records*
pub fn fight(records: &mut Vec<Match>, fire: &mut Chara, ice: &mut Chara, fire_id: usize, ice_id: usize)
-> FightCond {
    let mut choice: String = Default::default();
    loop {
        println!("-----------------------------");
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

        if choice.ends_with('1') {
            // I like left
            println!("Chose - {}!", fire.name.blue());
        } else if choice.ends_with('2') {
            // I like right
            game.res = 0.0;
            println!("Chose - {}!", ice.name.blue());
        } else if choice.ends_with('d') {
            // I dislike them both! (special value)
            game.res = 2.0;
            println!("Disliked both!");
        } else if choice.starts_with("end") {
            // End
            println!("Finishing rating period...");
            return FightCond::Last;
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

// Skill-based matchmaking? (best effort)
pub fn matchmake(rng: &mut ThreadRng, pool: &Vec<&mut Chara>, threshold: usize,
    unranked_picks: &mut HashSet<usize>)
-> Vec<usize> {
    let pool_size = pool.len();
    // when some character hasn't had a match yet, we ensure one of them is included
    // getting everyone off the start quickly
    let mut unranked_pool = pool.iter().enumerate().filter(|(n, a)| {
        a.hist.wins + a.hist.draw + a.hist.loss == 0 && !unranked_picks.contains(n)
    }).peekable();
    if unranked_pool.peek().is_some() {
        let one = unranked_pool.choose(rng).unwrap().0;
        let mut pair_id = rand::seq::index::sample(rng, pool_size, 1).into_vec();
        while pair_id[0] == one {
            pair_id[0] = rand::seq::index::sample(rng, pool_size, 1).index(0);
        }
        pair_id.push(one);
        unranked_picks.insert(one);
        return pair_id;
    }

    // regular matchmaker
    let mut pair_id = rand::seq::index::sample(rng, pool_size, 2).into_vec();
    let mut tries = 0;
    while tries < 20
          || (pair_id[1] as isize - pair_id[0] as isize).abs() > threshold as isize
    {
        pair_id = rand::seq::index::sample(rng, pool_size, 2).into_vec();
        tries += 1;
    }
    pair_id
}

// Parses line for tags (series, stages) and flags (pc98, notgirl, and nameless)
// Consumes: line
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

// Takes tags and generates the pool of contestants
// Consumes: line
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