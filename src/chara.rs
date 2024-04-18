// Functions relating to characters

use crate::{Chara, Glicko, Past, Match};
use std::collections::VecDeque;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

// Reset a character stat
pub fn reset(chara: &mut Chara) {
    *chara = Chara {
        rank: Glicko {
            rate: 1500.0,
            devi: 350.0,
            vola: 0.06,
        },
        hist: Past {
            wins: 0,
            loss: 0,
            draw: 0,
            old_rank: VecDeque::with_capacity(7),
            old_rate: VecDeque::with_capacity(7),
        },
        recent: VecDeque::with_capacity(7),
        ..chara.clone()
    };
}

// Updates the history records for each character
pub fn update_history(touhous: &mut Vec<Chara>, records: &Vec<Match>) {
    // fumos are references to touhous, right
    let mut fumos: Vec<&mut Chara> = Vec::with_capacity(touhous.len());
    for fumo in touhous.iter_mut() {
        fumos.push(fumo);
    }
    fumos.sort_by(|a, b| b.rank.rate.partial_cmp(&a.rank.rate).unwrap());

    let mut rank = 1;
    let mut last_rating = fumos[0].rank.rate;
    for fumo in fumos {
        // calculate rank
        if fumo.rank.rate < last_rating {
            rank += 1;
            last_rating = fumo.rank.rate;
        }
        // max 7 historical entries
        if fumo.hist.old_rank.len() >= 7 {
            fumo.hist.old_rank.pop_back();
            fumo.hist.old_rate.pop_back();
        }
        fumo.hist.old_rank.push_front(rank);
        fumo.hist.old_rate.push_front(fumo.rank.rate);
    }

    // update win/loss/draw and recent battles
    for battle in records.iter() {
        // wdl
        match battle.res {
            r if r == 2.0 => {
                touhous[battle.one].hist.loss += 1;
                touhous[battle.two].hist.loss += 1;
            },
            r if r == 1.0 => {
                touhous[battle.one].hist.wins += 1;
                touhous[battle.two].hist.loss += 1;
            },
            r if r == 0.5 => {
                touhous[battle.one].hist.draw += 1;
                touhous[battle.two].hist.draw += 1;
            },
            r if r == 0.0 => {
                touhous[battle.one].hist.loss += 1;
                touhous[battle.two].hist.wins += 1;
            },
            _ => { println!("update_history: ???"); },
        }
        // recent battles
        if touhous[battle.one].recent.len() >= 7 {
            touhous[battle.one].recent.pop_back();
        }
        if touhous[battle.two].recent.len() >= 7 {
            touhous[battle.two].recent.pop_back();
        }
        touhous[battle.one].recent.push_front(battle.clone());
        touhous[battle.two].recent.push_front(battle.clone());
    }
}

// Summons the mutable reference to the two characters at index1,2 (no bound check)
pub fn summon<'a>(touhous: &'a mut Vec<&mut Chara>, index1: usize, index2: usize)
-> (&'a mut Chara, &'a mut Chara, usize, usize) {
    if index1 > index2 {
        let (a, b) = touhous.split_at_mut(index1);
        (&mut *a[index2], &mut *b[0], index2, index1)
    } else {
        let (a, b) = touhous.split_at_mut(index2);
        (&mut *a[index1], &mut *b[0], index1, index2)
    }
}

// Find a character by name
pub fn find(touhous: &Vec<Chara>, query: String)
-> Option<&Chara> {
    let matcher = SkimMatcherV2::default();
    let mut best_match: Option<&Chara> = None;
    let mut best_score = 0;
    for th in touhous.iter() {
        if let Some(score) = matcher.fuzzy_match(&th.name, &query) {
            if score > best_score {
                best_match = Some(th);
                best_score = score;
            }
        }
    }
    best_match
}
pub fn find_mut(touhous: &mut Vec<Chara>, query: String)
-> Option<&mut Chara> {
    let matcher = SkimMatcherV2::default();
    let mut best_match: Option<&mut Chara> = None;
    let mut best_score = 0;
    for th in touhous.iter_mut() {
        if let Some(score) = matcher.fuzzy_match(&th.name, &query) {
            if score > best_score {
                best_match = Some(th);
                best_score = score;
            }
        }
    }
    best_match
}
// Exact match
pub fn find_mut_exact(touhous: &mut Vec<Chara>, query: String)
-> Option<&mut Chara> {
    for th in touhous.iter_mut() {
        if th.name == query {
            return Some(th);
        }
    }
    None
}