// Tohorank: tohosort but infinite and with numbers!
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
mod norm;
mod lobby;

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
    let mut data_path = dirs::home_dir().expect("Home directory");
    data_path.push(".tohorank");
    data_path.push("data.bin");
    // open the data file
    let data_file = match File::open(&data_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Data file not found! Creating a new one...");
            data::generate_data(&data_path);
            File::open(&data_path).unwrap() // surely can't be worse
        }
    };
    let reader = BufReader::new(data_file);
    // read all the touhous into memory
    let mut touhous: Vec<Chara> = match bincode::deserialize_from(reader) {
        Ok(ths) => ths,
        Err(_) => {
            println!("Data file not good! Creating a new one...");
            let mut backup_path = data_path.clone();
            backup_path.pop();
            backup_path.push("data.bin.bak");
            let _ = fs::copy(&data_path, &backup_path);
            println!("The original file saved at '{}'", backup_path.display());
            data::generate_data(&data_path);
            let data_file_again = File::open(&data_path).unwrap();
            let reader_again = BufReader::new(data_file_again);
            bincode::deserialize_from(reader_again).unwrap() // surely can't be worse
        }
    };
    let souls_onboard = touhous.len();
    let mut records: Vec<Match> = Vec::new();

    println!("Reading data file complete, got {} chracters.", souls_onboard);
    update_data(&mut touhous, &data_path); // why not auto-update

    let mut history_path = data_path.clone();
    history_path.pop();
    history_path.push("history.txt");

    println!("=========~ Tohorank: Lobby ~=========");
    lobby_help();

    let mut rl = DefaultEditor::new()?;
    rl.history_mut().set_max_len(100).ok();
    loop {
        rl.load_history(&history_path).ok();
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
                    let mut pair_id = sort::matchmake(&mut rng, &participants, &mut picks);
                    loop {
                        let (one, two) = chara::summon(&mut participants, &pair_id[0], &pair_id[1]);
                        match sort::fight(&mut records, one, two, indices[pair_id[0]], indices[pair_id[1]]) {
                            FightCond::Next => {
                                pair_id = sort::matchmake(&mut rng, &participants, &mut picks);
                            },
                            FightCond::Undo => {
                                // map global id (in records) -> participant id (for summon)
                                let (global_id1, global_id2) = (records.last().unwrap().one, records.last().unwrap().two);
                                let participant_id1 = indices.iter().position(|a| *a == global_id1).unwrap();
                                let participant_id2 = indices.iter().position(|a| *a == global_id2).unwrap();
                                pair_id = vec![participant_id1, participant_id2];
                                records.pop();
                            },
                            FightCond::Last => {
                                glicko::calc(&mut touhous, &records);
                                data::write_data(&touhous, &data_path);
                                records.clear();
                                println!("Data saved! Returning to lobby...");
                                break;
                            },
                        }
                    }
                } else if line.starts_with("l") {
                    // list!
                    let mut how_many = 25;
                    let mut name_filter = "".to_owned();
                    let mut tags_filter = "".to_owned();
                    for token in line.trim().split(" ").skip(1) {
                        if token.parse::<usize>().is_ok() {
                            how_many = token.parse().unwrap();  // is a number
                        } else {
                            // check if it is a tag
                            let token_unsigned = if token.starts_with("-") {
                                &token[1..]
                            } else {
                                &token[..]
                            };
                            match Tags::from_str(token_unsigned) {
                                Ok(_) => {
                                    // is a tag, add it to filter
                                    tags_filter.push_str(&(token.to_string() + " "));
                                },
                                Err(_) => {
                                    if token_unsigned.contains("pc98")
                                       || token_unsigned.contains("notgirl")
                                       || token_unsigned.contains("nameless")
                                    {
                                        tags_filter.push_str(&(token.to_string() + " "));
                                    } else {
                                        // is not a flag, treat as name
                                        name_filter.push_str(&(token.to_string() + " "));
                                    }
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
                    lobby::list(invited_immutable, how_many, name_filter.trim());
                } else if line.starts_with("stat") {
                    // stat!
                    match line.split_once(" ") {
                        Some((c, name)) => {
                            match chara::find(&touhous, name.to_string()) {
                                Some(th) => {
                                    lobby::stat(th, &touhous, c.len() > 4);
                                },
                                None => { println!("Character \"{}\" not found!", name); },
                            }
                        }
                        None => { println!("Usage: stat [character]"); },
                    }
                } else if line.starts_with("n") {
                    // compare rankings
                    let everyone = sort::bouncer("".to_string(), &mut touhous).0
                        .into_iter()
                        .map(|a| &*a)
                        .collect();
                    let list = norm::build(everyone, 0);
                    let correlation = norm::wrc_calc(&list, 2);
                    println!("Weighted Rank Correlation = {}",
                        format!("{:.3}", correlation).bold()
                    );
                    println!("The rankings {} {}.\n",
                        match correlation.abs() {
                            0.0 => "are ",
                            1.0 => "perfectly",
                            0.0..=0.2 => "weakly",
                            0.2..=0.4 => "",
                            0.4..=0.6 => "roughly",
                            0.6..=0.8 => "mostly",
                            0.8..=1.0 => "strongly",
                            _ => "make "
                        },
                        if correlation == 0.0 {
                            "not correlated"
                        } else if correlation.abs() > 1.0 {
                            "no sense"
                        } else if correlation < 0.0 {
                            "disagree"
                        } else {
                            "agree"
                        }
                    );

                    println!("--- More stats ---");
                    let correlation_unweighted = norm::wrc_calc(&list, 1);
                    println!("Unweighted (Spearman's Rho) = {:.3}", correlation_unweighted);
                    let t_score = correlation
                        * ((list.len() as f64 - 2.0) / (1.0 - correlation.powf(2.0))).sqrt();
                    println!("t-score: {:.3}", t_score);

                    norm::show_plot(&list);
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
                                        data::write_data(&touhous, &data_path);
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
                                    data::write_data(&touhous, &data_path);
                                },
                                None => { println!("Character \"{}\" not found!", name); },
                            }
                        }
                        None => { println!("Usage: know [character]"); },
                    }
                } else if line.starts_with("update") {
                    data::update_data(&mut touhous, &data_path);
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
                rl.save_history(&history_path).unwrap();
            }
            Err(ReadlineError::Interrupted) => {
                println!("Caught Ctrl-C, Exit");
                break;
            }
            Err(_) => { eprintln!("Error?"); }
        }
    } // end lobby loop

    Ok(())
}