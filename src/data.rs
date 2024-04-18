// Functions for manipulating the data file

use crate::{chara::find_mut_exact, Glicko, Past, groups::Tags, Chara};
use std::collections::HashSet;
use std::str::FromStr;
use std::collections::VecDeque;
use std::fs::File;
use std::time::SystemTime;
use std::io::{self, Write, BufRead, BufWriter};
use std::process;

// Reads a line from the stock list and give a character
pub fn chara_from_string(line: String)
-> Chara {
    // fields that depend on the list
    let mut chara_name: String = String::from("");
    let mut chara_groups: HashSet<Tags> = HashSet::new();
    let mut chara_flags: [bool; 4] = [false; 4];
    for (part, data) in line.split("; ").enumerate() {
        if part == 0 {
            // name
            chara_name = data.to_string();
        } else if part == 1 {
            // groups
            for group in data.split(" ") {
                chara_groups.insert(Tags::from_str(group).unwrap());
            }
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
    touhou
}

// Generate the data file from a stock list of characters
pub fn generate_data() {
    let start = SystemTime::now();
    // character array
    let mut characters: Vec<Chara> = Vec::with_capacity(163);

    // read from touhous.txt
    let mut err = false;
    let file = File::open("./src/touhous.txt").unwrap();
    let reader = io::BufReader::new(file);
    for (number, line) in reader.lines().skip(3).enumerate() {
        match line {
            Ok(l) => {
                let touhou = chara_from_string(l);
                println!("#{}: {}", number + 1, touhou.name);
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

// Update the data file to add (not remove) new characters and flags
pub fn update_data(touhous: &mut Vec<Chara>) {
    // todo: error handling
    let file = File::open("./src/touhous.txt").unwrap();
    let reader = io::BufReader::new(file);
    let (mut updated, mut added) = (0, 0);
    for (_, line) in reader.lines().skip(3).enumerate() {
        match line {
            Ok(l) => {
                if l.is_empty() {
                    continue;
                }
                let tokens: Vec<&str> = l.split("; ").collect();
                let name = tokens.get(0).unwrap();
                if let Some(th) = find_mut_exact(touhous, name.to_string()) {
                    // update
                    updated += 1;
                    (th.flags[0], th.flags[1], th.flags[2]) = (false, false, false);
                    for (part, data) in tokens.iter().enumerate() {
                        if part == 1 {
                            for tag in data.split(" ") {
                                th.groups.insert(Tags::from_str(tag).unwrap());
                            }
                        } else if part > 1 {
                            match *data {
                                "pc98" => {
                                    th.flags[0] = true;
                                },
                                "nameless" => {
                                    th.flags[1] = true;
                                },
                                "notgirl" => {
                                    th.flags[2] = true;
                                },
                                _ => { println!("???: Unknown flag.") },
                            }
                        }
                    }
                } else {
                    // create
                    added += 1;
                    let th = chara_from_string(l);
                    touhous.push(th);
                }
            }
            Err(error) => {
                eprintln!("\nError reading... {}", error);
                break;
            }
        }
    }
    write_data(touhous);
    println!("Update: {} characters updated, {} characters added.", updated, added);
}

// Write to the data file
pub fn write_data(touhous: &Vec<Chara>) {
    // serialize
    let encoded: Vec<u8> = bincode::serialize(touhous).unwrap();

    // save to data file
    let data_file = File::create("data.bin").unwrap();
    let mut writer = BufWriter::new(data_file);
    writer.write_all(&encoded).unwrap();
}