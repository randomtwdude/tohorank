// Tohorank: tohosort but infinite and with numbers!
// Version 0.1.0
// Borrow Checker(tm) approved.

use crate::data::update_data;
use crate::groups::Tags;
use std::fs::{self, File};
use std::collections::{HashSet, VecDeque};
use std::io::{self, BufReader, Write};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result, history::History};
use strum::IntoEnumIterator;

mod glicko;
mod groups;
mod stats;
mod data;
mod chara;
mod sort;

// Status returned by fight()
enum FightCond {
    Next,
    Last,
    Undo,
}

// Glicko ratings
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Glicko {
    rate: f64,
    devi: f64,
    vola: f64,
}

// Character's past records
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Past {
    wins: usize,
    loss: usize,
    draw: usize,
    old_rate: VecDeque<f64>,            // tracks the rating and rank some sessions ago
    old_rank: VecDeque<usize>,
    peak_rate: Option<(f64, String)>,   // peak rating and time
    peak_rank: Option<(usize, String)>,
}

// Character
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Chara {
    name:   String,             // display name
    rank:   Glicko,             // glicko rank information
    hist:   Past,               // historical stats
    recent: VecDeque<Match>,    // recent battles
    groups: HashSet<Tags>,      // groups this character belongs to
    flags:  [bool; 4]           // True if: PC98, nameless, not a girl, don't know them
                                // use the methods for checks
}

impl Chara {
    // queries
    fn is_pc98(&self) -> bool {
        self.flags[0]
    }
    fn is_nameless(&self) -> bool {
        self.flags[1]
    }
    fn is_not_girl(&self) -> bool {
        self.flags[2]
    }
    // characters marked "don't know" are hidden in rankings
    fn dont_know(&self) -> bool {
        self.flags[3]
    }
    fn toggle_dont_know(&mut self) {
        self.flags[3] = !self.flags[3];
    }
    // tag filtering
    fn has_tag(&self, tag: &Tags) -> bool {
        self.groups.contains(tag)
    }
}

// A matchup between two characters
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Match {
    one: usize, // the global IDs used to address THE Vec<Chara>
    two: usize,
    res: f32,
}

const INCLUSIVE: bool = true; // for handling tags
const DEVIATION_BAR: f64 = 160.0; // threshold for "high deviation"

fn lobby_help() {
    println!("-- 'start':   start a new session.");
    println!("-- 'list':    show the ranking list.");
    println!("-- 'stat':    see stats of a character.");
    println!("   'stat!':   even more stats!");
    println!("-------------------------------------");
    println!("-- 'reset':   reset the stats of a character.");
    println!("-- 'know':    hide/unhide a character in rankings.");
    println!("-- 'update':  updates the data file");
    println!("-- 'help':    display this message.");
    println!("-- 'tags':    display a list of filters");
    println!("-- 'exit':    See you next time.");
}

fn main()
-> Result<()> {
    let mut rng = rand::thread_rng();
    // open the data file
    let data_file = match File::open("data.bin") {
        Ok(file) => file,
        Err(_) => {
            println!("Data file not found! Creating a new one...");
            data::generate_data();
            File::open("data.bin").unwrap() // surely can't be worse
        }
    };
    let reader = BufReader::new(data_file);
    // read all the touhous into memory
    let mut touhous: Vec<Chara> = match bincode::deserialize_from(reader) {
        Ok(ths) => ths,
        Err(_) => {
            println!("Data file not good! Creating a new one...");
            let _ = fs::copy("data.bin", "data.bin.bak");
            println!("The original file saved as 'data.bin.bak'");
            data::generate_data();
            let data_file_again = File::open("data.bin").unwrap();
            let reader_again = BufReader::new(data_file_again);
            bincode::deserialize_from(reader_again).unwrap() // surely can't be worse
        }
    };
    let souls_onboard = touhous.len();
    let mut records: Vec<Match> = Vec::new();

    println!("Reading data file complete, got {} chracters.", souls_onboard);
    update_data(&mut touhous); // why not auto-update

    println!("=========~ Tohorank: Lobby ~=========");
    lobby_help();

    let mut rl = DefaultEditor::new()?;
    loop {
        rl.load_history("history.txt").ok();
        rl.history_mut().set_max_len(100).ok();
        let readline = rl.readline("Lobby >> ");

        match readline {
            Ok(line) => {
                if line.starts_with("star") {
                    // fight!
                    let filter_str = match line.trim().split_once(" ") {
                        Some((_, f)) => { f.trim().to_owned() },
                        None => String::from_str("").unwrap(),
                    };
                    // indices: global ID of participants (relative to the entire character vector)
                    let (mut participants, indices) = sort::bouncer(filter_str, &mut touhous);
                    if participants.len() < 2 {
                        println!("Cannot start with fewer than 2 participants!");
                        continue;
                    }
                    println!("{}",
                        format!("=== Starting a new session with {} characters... ===", participants.len()).blue()
                    );
                    // keep track of the players picked because they haven't gotten a chance yet
                    // so we don't keep picking them (the record is only written after this session ends)
                    let mut picks: HashSet<usize> = HashSet::with_capacity(participants.len());
                    let mut pair_id = sort::matchmake(&mut rng, &participants, 500, &mut picks);
                    loop {
                        let (one, two, id_one, id_two) = chara::summon(&mut participants, pair_id[0], pair_id[1]);
                        match sort::fight(&mut records, one, two, indices[id_one], indices[id_two]) {
                            FightCond::Next => {
                                pair_id = sort::matchmake(&mut rng, &participants, 500, &mut picks);
                            }
                            FightCond::Undo => {
                                // map global id (in records) -> participant id (for summon)
                                let (global_id1, global_id2) = (records.last().unwrap().one, records.last().unwrap().two);
                                let participant_id1 = indices.iter().position(|a| *a == global_id1).unwrap();
                                let participant_id2 = indices.iter().position(|a| *a == global_id2).unwrap();
                                pair_id = vec![participant_id1, participant_id2];
                                records.pop();
                            }
                            FightCond::Last => {
                                glicko::calc(&mut touhous, &records);
                                data::write_data(&touhous);
                                records.clear();
                                println!("Data saved! Returning to lobby...");
                                break;
                            }
                        }
                    }
                } else if line.starts_with("l") {
                    // list!
                    let mut how_many = 25;
                    let mut name_filter = "".to_owned();
                    let mut tags_filter = "".to_owned();
                    for token in line.trim().split(" ").skip(1) {
                        if token.parse::<usize>().is_ok() {
                            // is a number
                            how_many = token.parse().unwrap();
                        } else {
                            // check if it is a tag
                            let token_unsigned = if token.starts_with("-") {
                                &token[1..]
                            } else {
                                &token[..]
                            };
                            // todo: flags should work too
                            match Tags::from_str(token_unsigned) {
                                Ok(_) => {
                                    // is a tag, add it to filter
                                    tags_filter.push_str(&(token.to_string() + " "));
                                },
                                Err(_) => {
                                    // is not a tag, treat as name
                                    name_filter.push_str(&(token.to_string() + " "));
                                },
                            }
                        }
                    }
                    // drop the indices since we don't need it here
                    let (invited, _) = sort::bouncer(tags_filter, &mut touhous);
                    if invited.len() == 0 {
                        println!("There's no one here... :(");
                        continue;
                    }
                    let invited_immutable: Vec<&Chara> =
                        invited.into_iter().map(|a| &*a).collect();
                    list(invited_immutable, how_many, name_filter.trim());
                } else if line.starts_with("stat") {
                    // stat!
                    match line.split_once(" ") {
                        Some((c, name)) => {
                            match chara::find(&touhous, name.to_string()) {
                                Some(th) => {
                                    stat(th, &touhous, c.len() > 4);
                                },
                                None => { println!("Character \"{}\" not found!", name); },
                            }
                        }
                        None => { println!("Usage: stat [character]"); },
                    }
                } else if line.starts_with("h") {
                    lobby_help();
                } else if line.starts_with("reset") {
                    match line.split_once(" ") {
                        Some((_, name)) => {
                            match chara::find_mut(&mut touhous, name.to_string()) {
                                Some(th) => {
                                    println!("{}: You are about to RESET the ratings and historical stats of {}.",
                                        "WARNING".red(),
                                        th.name.red()
                                    );
                                    println!("Type 'YES' in uppercase to confirm...");
                                    let _ = io::stdout().flush();
                                    let mut choice = String::default();
                                    let _ = io::stdin().read_line(&mut choice);
                                    if choice == "YES\n" {
                                        chara::reset(th);
                                        println!("Resetting {}...", th.name);
                                        data::write_data(&touhous);
                                    } else {
                                        println!("Aborted.");
                                    }
                                },
                                None => { println!("Character \"{}\" not found!", name); },
                            }
                        }
                        None => { println!("Usage: reset [character]"); },
                    }
                } else if line.starts_with("know") {
                    // don't know status hides a character from the rankings
                    match line.split_once(" ") {
                        Some((_, name)) => {
                            match chara::find_mut(&mut touhous, name.to_string()) {
                                Some(th) => {
                                    th.toggle_dont_know();
                                    println!("{} will {}be hidden.",
                                        th.name.bold(),
                                        if th.dont_know() {
                                            ""
                                        } else {
                                            "no longer "
                                        }
                                    );
                                    data::write_data(&touhous);
                                },
                                None => { println!("Character \"{}\" not found!", name); },
                            }
                        }
                        None => { println!("Usage: know [character]"); },
                    }
                } else if line.starts_with("update") {
                    data::update_data(&mut touhous);
                } else if line.starts_with("tags") {
                    println!("List of all filter tags:");
                    println!("Add '-' to the front to exclude them instead.");
                    println!("You can also specify by the number like 'th06'\n");
                    for tag in Tags::iter() {
                        if tag.is_series_tag() {
                            println!("{:<8}{} ~ {}", format!("{:?}", tag), tag.exname(), tag.name());
                        } else {
                            println!("{:<8}{}", format!("{:?}", tag), tag.name());
                        }
                    }
                    println!("pc98");
                    println!("notgirl");
                    println!("nameless");
                } else if line.starts_with("e") {
                    break;
                } else {
                    println!("?");
                }
                let _ = rl.add_history_entry(line);
                rl.save_history("history.txt").unwrap();
            }
            Err(ReadlineError::Interrupted) => {
                println!("Caught Ctrl-C, Exit");
                break;
            }
            Err(_) => { eprintln!("Error?"); }
        }
    } // end lobby loop

    Ok(()) // ok

}

// Show detailed stats about a character
fn stat(chara: &Chara, touhous: &Vec<Chara>, full_rankings: bool) {
    println!("{:-<1$}", "", 50);
    // Name and overall rank
    let no_tags: Vec<(Tags, bool)> = vec![];
    let everyone = stats::filter_group(no_tags.clone(), touhous);
    let ranking_overall = stats::rank_in_group(chara, &everyone);
    println!("{0: <45}{1: >13}",
        format!("~~ {} ~~", chara.name.bold()),
        format!("Rank #{}/{}", ranking_overall.0, ranking_overall.1)
    );
    println!("{:-<1$}", "", 50);

    // Rating information
    println!("==> {}", "RATING".bold());
    println!("{}",
        if chara.rank.devi > DEVIATION_BAR {
            format!("    {} ± {1:.0} | (volatility: {2:.6})",
                format!("{:.2}", chara.rank.rate).bold(),
                chara.rank.devi * 1.96,
                chara.rank.vola
            ).truecolor(182, 185, 191)
        } else {
            format!("    {} ± {1:.0} | (volatility: {2:.6})",
                format!("{:.2}", chara.rank.rate).bold(),
                chara.rank.devi * 1.96,
                chara.rank.vola
            ).truecolor(140, 180, 250)
        }
    );
    if chara.rank.devi > DEVIATION_BAR {
        println!("    ⓘ The uncertainty is high, do more battles!\n");
    }

    if !chara.hist.old_rank.is_empty() {
        println!("    -- Last {} {} --",
            chara.hist.old_rank.len(),
            if chara.hist.old_rank.len() > 1 {
                "sessions"
            } else {
                "session"
            }
        );
        let pt_diff = chara.rank.rate - *chara.hist.old_rate.back().unwrap();
        let rk_diff: isize = ranking_overall.0 as isize - *chara.hist.old_rank.back().unwrap() as isize;
        println!("    {} {:.0} {} {}.",
            if pt_diff > 0.0 {
                "🡽".blue()
            } else if pt_diff == 0.0 {
                "🡺".white()
            } else {
                "🡾".red()
            },
            pt_diff.abs(),
            if pt_diff == 1.0 {
                "point"
            } else {
                "points"
            },
            if pt_diff >= 0.0 {
                "gained".blue()
            } else {
                "lost".red()
            }
        );
        println!("    {} {} {} {}.",
            if rk_diff < 0 {
                "🡽".blue()
            } else if rk_diff == 0 {
                "🡺".white()
            } else {
                "🡾".red()
            },
            rk_diff.abs(),
            if rk_diff.abs() == 1 {
                "place"
            } else {
                "places"
            },
            if rk_diff <= 0 {
                "gained".blue()
            } else {
                "lost".red()
            }
        );
    }
    // Peaks
    if let Some((prk, prk_time)) = &chara.hist.peak_rank {
        println!("\n    Highest rank: #{} on {}", prk, prk_time);
    }
    if let Some((prt, prt_time)) = &chara.hist.peak_rate {
        println!("    Highest rating: {:.0} on {}", prt, prt_time);
    }


    // Rank informations
    println!("\n==> {}", "RANKINGS".bold());
    // Overall ranks
    stats::print_rank_in_group(chara, no_tags, touhous);
    // All the other ranks
    if full_rankings {
        for tag in chara.groups.iter() {
            stats::print_rank_in_group(chara, vec![(tag.clone(), INCLUSIVE)], touhous);
        }
    } else {
        println!("\n    ⓘ For rankings in various works, use `stat!`");
    }

    // Stats
    println!("\n==> {}", "STATISTICS".bold());
    let total = chara.hist.wins + chara.hist.draw + chara.hist.loss;
    println!("    Wins:   {} ({}%)",
        chara.hist.wins,
        if total == 0 {
            0
        } else {
            100 * chara.hist.wins / total
        }
    );
    println!("    Draws:  {}", chara.hist.draw);
    println!("    Losses: {}", chara.hist.loss);

    // Recent battles
    if !chara.recent.is_empty() {
        println!("\n==> {}", "RECENT BATTLES".bold());
    }
    for battle in chara.recent.iter() {
        let side;
        if touhous.get(battle.one).unwrap().name == chara.name {
            side = 1;
        } else {
            side = 2;
        }
        println!("    {} against {} ({:.0})",
            match battle.res {
                r if r == 0.5 => { "Drew".white().bold() },
                r if r == 2.0 => { "Drew (lost)".red().bold() },
                r if r == 0.0 && side == 1 => { "Lost".red().bold() },
                r if r == 0.0 && side == 2 => { "Won".blue().bold() },
                r if r == 1.0 && side == 1 => { "Won".blue().bold() },
                r if r == 1.0 && side == 2 => { "Lost".red().bold() },
                _ => { "?".red().bold() }
            },
            if side == 1 {
                touhous.get(battle.two).unwrap().name.clone()
            } else {
                touhous.get(battle.one).unwrap().name.clone()
            },
            if side == 1 {
                touhous.get(battle.two).unwrap().rank.rate
            } else {
                touhous.get(battle.one).unwrap().rank.rate
            }
        );
    }
    println!();
}

// Show the current rankings up to *first* entries
fn list(mut touhous: Vec<&Chara>, first: usize, name_filter: &str) {
    println!("------------------------------------------------------");
    println!("#    Name                      Rating           Extra ");
    println!("------------------------------------------------------");

    touhous.sort_by(|a, b| b.rank.rate.partial_cmp(&a.rank.rate).unwrap());

    let mut rank = 1;
    let mut count = 0;
    let mut last_rating = touhous[0].rank.rate;
    for (n, touhou) in touhous.iter().filter(|t| !t.dont_know()).enumerate() {
        if count >= first {
            let mut left = 0;
            for more_touhou in touhous[n..].iter() {
                if more_touhou.rank.rate != last_rating {
                    break;
                }
                left += 1;
            }
            if left > 0 {
                println!("... {} more characaters with the same rank.", left);
            }
            break;
        }
        if touhou.rank.rate < last_rating {
            rank = n + 1;
            last_rating = touhou.rank.rate;
        }
        // filter by name
        if !touhou.name.to_lowercase().contains(&name_filter.to_lowercase()) {
            continue;
        }
        // print entry
        count += 1;
        // trend is wrong if any filtering is applied
        let rk_diff: isize = if touhou.hist.old_rank.len() > 0 {
            rank as isize - *touhou.hist.old_rank.back().unwrap() as isize
        } else {
            0
        };
        let trend = format!("{}",
            if rk_diff.abs() > 5 {
                format!("{}",
                    if rk_diff.is_positive() {
                        "🡾 ".red()
                    } else {
                        "🡽 ".blue()
                    }
                )
            } else {
                "".to_string()
            }
        );
        // we want to see if they're number 1 in any groups
        let mut favorite = "";
        let mut touhous2: Vec<Chara> = Vec::new();
        for th in touhous.iter() {
            #[allow(suspicious_double_ref_op)]
            touhous2.push(th.clone().clone()); // works
        }
        for tag in touhou.groups.iter() {
            let group = stats::filter_group(vec![(tag.clone(), INCLUSIVE)], &touhous2);
            if stats::rank_in_group(touhou, &group).0 == 1 {
                favorite = "★ ";
                break;
            }
        }
        // final entry
        let entry = format!("{:<4} {:<26}{}  {}{}",
            format!("{}.", rank),
            touhou.name,
            if touhou.rank.devi > DEVIATION_BAR {
                format!("({0: <7} ± {1:.0})",
                    format!("{:.2}", touhou.rank.rate).bold(),
                    touhou.rank.devi * 1.96
                ).truecolor(182, 185, 191)
            } else {
                format!("({0: <7} ± {1:.0})",
                    format!("{:.2}", touhou.rank.rate).bold(),
                    touhou.rank.devi * 1.96
                ).normal()
            },
            trend,
            favorite.blue()
        );
        match rank {
            1 => { println!("{}", entry.truecolor(245, 212, 95)); },
            2 => { println!("{}", entry.truecolor(180, 245, 212)); },
            3 => { println!("{}", entry.truecolor(240, 140, 95)); },
            _ => { println!("{}", entry); },
        }
    }
    println!();
}