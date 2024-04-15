// Tohorank: tohosort but infinite and with numbers!
// it runs a Glicko-2 tournament on the characters.

// terrible Rust code ahead
use std::process;
use std::fs::{self, File};
use std::time::SystemTime;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead, BufReader, Write, BufWriter};
use std::str::FromStr;
use rand::thread_rng;
use serde::{Serialize, Deserialize};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use crate::groups::Tags;

mod glicko;
mod groups;
mod stats;

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
    old_rate: VecDeque<f64>,   // tracks the rating and rank some sessions ago
    old_rank: VecDeque<usize>,
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
    // construct
    fn new(name: String, rank: Glicko, hist: Past, recent: VecDeque<Match>, groups: HashSet<Tags>, flags: [bool; 4])
    -> Result<Self> {
        Ok(Chara { name, rank, hist, recent, groups, flags })
    }
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
    // characters marked "don't know" are hidden
    fn dont_know(&self) -> bool {
        self.flags[3]
    }
    fn set_dont_know(&mut self) {
        self.flags[3] = true;
    }
    fn unset_dont_know(&mut self) {
        self.flags[3] = false;
    }
    // tag filtering
    fn has_tag(&self, tag: &Tags) -> bool {
        self.groups.contains(tag)
    }
}

// A matchup between two characters
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Match {
    one: usize,
    two: usize,
    res: f32,
}

// Generate the data file from a stock list of characters
fn generate_data() {
    let start = SystemTime::now();
    // character array
    let mut characters: Vec<Chara> = Vec::with_capacity(163);

    // read from touhous.txt
    let mut err = false;
    let file = File::open("./src/touhous.txt").unwrap();
    let reader = io::BufReader::new(file);
    for (number, line) in reader.lines().skip(3).enumerate() {
        match line {
            Ok(input) => {
                // fields that depend on the list
                let mut chara_name: String = String::from("");
                let mut chara_groups: HashSet<Tags> = HashSet::new();
                let mut chara_flags: [bool; 4] = [false; 4];
                for (part, data) in input.split("; ").enumerate() {
                    if part == 0 {
                        // name
                        print!("#{}: {}", number + 1, data);
                        chara_name = data.to_string();
                    } else if part == 1 {
                        // groups
                        for group in data.split(" ") {
                            chara_groups.insert(Tags::from_str(group).unwrap());
                        }
                        println!();
                    } else {
                        // flags
                        match data {
                            "pc98" => {
                                chara_flags[0] = true;
                            },
                            "nameless" => {
                                chara_flags[1] = true;
                            },
                            "notgirl" => {
                                chara_flags[2] = true;
                            },
                            _ => { println!("???: Unknown flag.") },
                        }
                    }
                }
                let touhou = Chara {
                    name: chara_name,
                    rank: Glicko {
                        rate: 1500.0,
                        devi: 350.0,
                        vola: 0.06,
                    },
                    hist: Past {
                        wins: 0,
                        loss: 0,
                        draw: 0,
                        old_rate: VecDeque::with_capacity(7),
                        old_rank: VecDeque::with_capacity(7),
                    },
                    recent: VecDeque::with_capacity(7),
                    groups: chara_groups,
                    flags: chara_flags,
                };
                characters.push(touhou);
            }
            Err(error) => {
                eprintln!("\nError reading... {}", error);
                err = true;
                break;
            }
        }
    }
    write_data(&characters);
    if err {
        println!("Data file generation INCOMPLETE! Something's gone wrong.");
        process::exit(1);
    } else {
        println!("==> Data file generation completed.");
        println!("    got {} characters in {} µs.",
            characters.len(),
            start.elapsed().unwrap().as_micros()
        );
    }
}

// Update the data file
fn write_data(touhous: &Vec<Chara>) {
    // serialize
    let encoded: Vec<u8> = bincode::serialize(touhous).unwrap();

    // save to data file
    let data_file = File::create("data.bin").unwrap();
    let mut writer = BufWriter::new(data_file);
    writer.write_all(&encoded).unwrap();
}

// One battle, may add an entry to *records*
fn fight(records: &mut Vec<Match>, fire: &mut Chara, ice: &mut Chara, fire_id: usize, ice_id: usize)
-> FightCond {
    let mut choice: String = Default::default();
    loop {
        println!("-----------------------------");
        // println!("Battle #{}: {} vs {}", records.len() + 1, fire.name, ice.name);
        println!("Battle #{}: {} ({:.0}) vs {} ({:.0})",
            records.len() + 1,
            fire.name, fire.rank.rate,
            ice.name, ice.rank.rate
        );
        print!("Pick [ 'h' for help ] >> ");
        let _ = io::stdout().flush();
        choice.clear();
        let _ = io::stdin().read_line(&mut choice);

        let mut game = Match {
            one: fire_id,
            two: ice_id,
            res: 1.0,
        };

        if choice.starts_with('1') {
            // I like left
            println!("Chose - {}!", fire.name.blue());
        } else if choice.starts_with('2') {
            // I like right
            game.res = 0.0;
            println!("Chose - {}!", ice.name.blue());
        } else if choice.starts_with('d') {
            // I dislike them both! (special value)
            game.res = 2.0;
            println!("Disliked both!");
        } else if choice.starts_with('e') {
            // End
            println!("Finishing rating period...");
            return FightCond::Last;
        } else if choice.starts_with('u') {
            // Undo
            if records.len() == 0 {
                println!("This is the first battle!");
                continue;
            }
            println!("Going back...");
            return FightCond::Undo;
        } else if choice.starts_with('h') {
            // Help
            println!("1/2 to choose left/right");
            println!("<Enter> for draws");
            println!("d if you DISLIKE BOTH of them");
            println!("u to undo");
            println!("e to end this session");
            continue;
        } else {
            game.res = 0.5;
            println!("Chose - Draw!");
        }

        records.push(game);
        return FightCond::Next;
    }
}

// Updates the history records for each character
fn update_history(touhous: &mut Vec<Chara>) {
    // pre-calculates the rank of everyone
    // because we can't do this in the for-loop below
    //              -- the Borrow Checker
    let mut mirror = touhous.clone();
    mirror.sort_by(|a, b| a.rank.rate.partial_cmp(&b.rank.rate).unwrap());
    mirror.reverse();
    // TODO: put a hashmap here i guess

    // Actually update
    for th in touhous.iter_mut() {
        // only keep 7 records
        if th.hist.old_rank.len() == 7 {
            th.hist.old_rank.pop_back();
            th.hist.old_rate.pop_back();
        }
        // insert new records
        th.hist.old_rate.push_front(th.rank.rate);
    }
}

// Updates all ratings, write_data() after use.
fn glicko_calc(touhous: &mut Vec<Chara>, records: &Vec<Match>) {
    println!("Tallying {} matches...", records.len());
    update_history(touhous);

    // transform to the glicko-2 scale
    for th in touhous.iter_mut() {
        glicko::glicko_two_scale(&mut th.rank.rate, &mut th.rank.devi);
    }

    // First we need to calculate the quantities v and delta
    let mut qt_v: HashMap<usize, f64> = HashMap::new();
    let mut qt_d: HashMap<usize, f64> = HashMap::new();

    // initialize hashmaps
    for battle in records.iter() {
        qt_v.insert(battle.one, 0.0);
        qt_v.insert(battle.two, 0.0);
        qt_d.insert(battle.one, 0.0);
        qt_d.insert(battle.two, 0.0);
    }

    for battle in records.iter() {
        // fetch numbers
        let r1 = touhous[battle.one].rank.rate;
        let r2 = touhous[battle.two].rank.rate;
        let rd1 = touhous[battle.one].rank.devi;
        let rd2 = touhous[battle.two].rank.devi;
        let (s1, s2) = if battle.res == 2.0 {
            // both sides lose, but not as much as when only one side loses
            (0.25, 0.25)
        } else {
            (battle.res, 1.0 - battle.res)
        };
        // update v1
        let v1_add: f64 = glicko::part_v(&r1, &r2, &rd2);
        if let Some(v1) = qt_v.get_mut(&battle.one) {
            *v1 += v1_add;
        }
        // update v2
        let v2_add: f64 = glicko::part_v(&r2, &r1, &rd1);
        if let Some(v2) = qt_v.get_mut(&battle.two) {
            *v2 += v2_add;
        }
        // update d1
        let d1_add: f64 = glicko::part_d(&r1, &r2, &rd2, &s1);
        if let Some(d1) = qt_d.get_mut(&battle.one) {
            *d1 += d1_add;
        }
        // update d2
        let d2_add: f64 = glicko::part_d(&r2, &r1, &rd1, &s2);
        if let Some(d2) = qt_d.get_mut(&battle.two) {
            *d2 += d2_add;
        }
    }
    // finally, take the inverse of v
    for (_, val) in qt_v.iter_mut() {
        *val = val.powi(-1);
    }
    // and multiple d by v
    for (key, val) in qt_d.iter_mut() {
        *val *= qt_v[key];
    }

    // now we have the v and deltas, we move to calculating
    // the new rating volatilities

    // lower for better accuracy? (doesn't look like it)
    const CONV_TOLERANCE: f64 = 0.000001;
    // the system constant, the paper says it should be 0.3~1.2
    const TAU: f64 = 0.75;

    // update the volatility for all characters in this session
    for th in qt_v.keys() {
        touhous[*th].rank.vola = glicko::calc_new_volatility(
            &qt_v[th],
            &qt_d[th],
            &touhous[*th].rank.vola,
            &touhous[*th].rank.devi,
            &TAU,
            &CONV_TOLERANCE
        );
    }

    // now we update the rating deviations,
    // first round on all characters
    for th in touhous.iter_mut() {
        th.rank.devi = glicko::adjust_deviation(
            &th.rank.devi,
            &th.rank.vola
        );
    }
    // second round on battled characters
    for th in qt_v.keys() {
        touhous[*th].rank.devi = glicko::calc_new_deviation(
            &touhous[*th].rank.devi,
            &qt_v[th]
        );
    }

    // finally, we calculate the new ratings
    for th in qt_v.keys() {
        touhous[*th].rank.rate = glicko::calc_new_rating(
            &touhous[*th].rank.rate,
            &touhous[*th].rank.devi,
            &qt_v[th],
            &qt_d[th]
        );
    }

    // transform back to glicko scale
    for th in touhous.iter_mut() {
        glicko::glicko_one_scale(&mut th.rank.rate, &mut th.rank.devi);
    }
}

// Summons the mutable reference to the two characters at index1,2 (no bound check)
fn summon(touhous: &mut Vec<Chara>, index1: usize, index2: usize)
-> (&mut Chara, &mut Chara, usize, usize) {
    if index1 > index2 {
        let (a, b) = touhous.split_at_mut(index1);
        (&mut a[index2], &mut b[0], index2, index1)
    } else {
        let (a, b) = touhous.split_at_mut(index2);
        (&mut a[index1], &mut b[0], index1, index2)
    }
}

// Find a character by name
// TODO: some characters with short names (like Mai) are unreachable
fn find(touhous: &Vec<Chara>, query: String) -> Option<&Chara> {
    for th in touhous.iter() {
        if th.name.to_lowercase().contains(&query.to_lowercase()) {
            return Some(th);
        }
    }
    None
}

// Show detailed stats about a character
fn stat(chara: &Chara, touhous: &Vec<Chara>) {
    // Name and overall rank
    let ranking_overall = stats::rank_in_group(chara, touhous);
    println!("{0: <45}{1: >13}",
        format!("~~ {} ~~", chara.name.bold()),
        format!("Rank #{}/{}", ranking_overall.0, ranking_overall.1)
    );
    println!("{:-<1$}", "", 50);

    // Rating information
    println!("{} - Glicko-2", "RATING".bold());
    println!("{}",
        if chara.rank.devi > 120.0 {
            format!("    {0:.2} ± {1:.0} | (volatility: {2:.6})",
                chara.rank.rate,
                chara.rank.devi,
                chara.rank.vola
            ).truecolor(182, 185, 191)
        } else {
            format!("    {0:.2} ± {1:.0} | (volatility: {2:.6})",
                chara.rank.rate,
                chara.rank.devi,
                chara.rank.vola
            ).truecolor(140, 180, 250)
        }
    );
    if chara.rank.devi > 120.0 {
        println!("    ⓘ The uncertainty is high, do more battles!");
    }

    println!("\n");
    if !chara.hist.old_rank.is_empty() {
        println!("    -- Last {} sessions --", chara.hist.old_rank.len());
        let pt_diff = chara.rank.rate - *chara.hist.old_rate.back().unwrap();
        let rk_diff: isize = ranking_overall.0 as isize - *chara.hist.old_rank.back().unwrap() as isize;
        println!("{:+.0}", pt_diff);
        println!("{}", -rk_diff);
    }

}

// Show the current rankings up to *first* entries
fn list(mut touhous: Vec<Chara>, first: usize) {
    println!("====================== Ranking list ======================");
    println!("#    Name                      Rating           Volatility");
    println!("----------------------------------------------------------");

    touhous.sort_by(|a, b| a.rank.rate.partial_cmp(&b.rank.rate).unwrap());
    touhous.reverse();

    let mut rank = 1;
    let mut last_rating = touhous[0].rank.rate;
    for (count, touhou) in touhous.iter().enumerate() {
        if count >= first {
            let mut left = 0;
            for more_touhou in touhous[count..].iter() {
                if more_touhou.rank.rate != last_rating {
                    break;
                }
                left += 1;
            }
            if left > 0 {
                println!("... {} more characaters with the same rank.\n", left);
            }
            break;
        }
        if touhou.rank.rate < last_rating {
            rank += 1;
            last_rating = touhou.rank.rate;
        }
        println!("{0: <4} {1: <26}({2} ± {3:.0})  {4:.6}",
            format!("{}.", rank),
            touhou.name,
            format!("{0:.2}", touhou.rank.rate).bold(),
            touhou.rank.devi * 1.96, // 95% confidence
            touhou.rank.vola
        );
    }
}

fn main()
-> Result<()> {
    let mut rng = rand::thread_rng();
    // read data
    let data_file = match File::open("data.bin") {
        Ok(file) => file,
        Err(_) => {
            println!("Data file not found! Creating a new one...");
            generate_data();
            File::open("data.bin").unwrap() // surely can't be worse
        }
    };
    let reader = BufReader::new(data_file);
    // all the touhous
    let mut touhous: Vec<Chara> = match bincode::deserialize_from(reader) {
        Ok(ths) => ths,
        Err(_) => {
            println!("Data file not good! Creating a new one...");
            let _ = fs::copy("data.bin", "data.bin.bak");
            println!("The original file has been backed up as 'data.bin.bak'");
            generate_data();
            let data_file_again = File::open("data.bin").unwrap();
            let reader_again = BufReader::new(data_file_again);
            bincode::deserialize_from(reader_again).unwrap() // surely can't be worse
        }
    };
    let souls_onboard = touhous.len();
    // match records
    let mut records: Vec<Match> = Vec::new();

    println!("Reading data file complete, got {} touhous.", souls_onboard);

    println!("====== Tohorank: Lobby ======");
    println!("-- 'start': start a new session.");
    println!("-- 'list':  show the ranking list.");
    println!("-- 'stat':  see detailed stats about a character.");
    println!("-- 'info':  info about the rating system.");
    println!("-- 'exit':  exits");

    loop {
        // I don't know how to get history
        let mut rl = DefaultEditor::new()?;
        let readline = rl.readline("Lobby >> ");

        match readline {
            Ok(line) => {
                if line.starts_with("star") {
                    let mut pair_id = rand::seq::index::sample(&mut rng, souls_onboard, 2).into_vec();
                    loop {
                        let (one, two, id_one, id_two) = summon(&mut touhous, pair_id[0], pair_id[1]);
                        match fight(&mut records, one, two, id_one, id_two) {
                            FightCond::Next => {
                                pair_id = rand::seq::index::sample(&mut rng, souls_onboard, 2).into_vec();
                            }
                            FightCond::Undo => {
                                pair_id = vec![records[records.len() - 1].one, records[records.len() - 1].two];
                                records.pop();
                            }
                            FightCond::Last => {
                                glicko_calc(&mut touhous, &records);
                                write_data(&touhous);
                                records.clear();
                                println!("Data saved! Returning to lobby...");
                                break;
                            }
                        }
                    }
                } else if line.starts_with("l") {
                    let words: Vec<&str> = line.split(" ").collect();
                    let how_many = if words.len() > 1 {
                        words[1].trim().parse().unwrap()
                    } else {
                        10
                    };
                    list(touhous.clone(), how_many);
                } else if line.starts_with("stat") {
                    match line.split_once(" ") {
                        Some((_, name)) => {
                            match find(&touhous, name.to_string()) {
                                Some(th) => { stat(th, &touhous); },
                                None => { println!("Character \"{}\" not found!", name); },
                            }
                        }
                        None => { println!("Usage: stat [character]"); },
                    }
                } else if line.starts_with("i") {
                    glicko::glicko_info();
                } else if line.starts_with("e") {
                    break;
                } else {
                    println!("?");
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Caught Ctrl-C, Exit");
                break;
            }
            Err(_) => { eprintln!("?"); }
        }
    } // end lobby loop

    Ok(()) // ok
}
