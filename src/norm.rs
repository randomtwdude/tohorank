// Functions about the ranking comparisons

// Implemented here is a modification of the Spearman's rank correlation
// with weights, biased towards the top of the list.

// Only the intersection of the two rankings are compared.

// From the paper:
// A General Class of Weighted Rank Correlation Measures
// arxiv.org/pdf/2001.07298

use std::collections::HashMap;
use std::io::{BufReader, BufRead};
use crate::File;
use termplot::*;

use crate::Chara;

// Build the list containing the characters (sorted by rank)
// and their ranks in the target rankings (to be compared to)
// target does nothing for now
pub fn build(mut touhous: Vec<&Chara>, target: u8)
-> HashMap<String, (usize, usize)> {

    // open the file (just testing for now)
    let ref_list = File::open("./src/best_taste.txt").expect("Run in src");

    let mut list: HashMap<String, (usize, usize)> = HashMap::with_capacity(touhous.len());

    // add all the characters to the list
    // sort by rank
    touhous.sort_by(|a, b| b.rank.rate.partial_cmp(&a.rank.rate).unwrap());
    // set the rank in target to 0 for now
    for (n, th) in touhous.iter().enumerate() {
        list.insert(th.name.clone(), (n + 1, 0));
    }

    // add from the target list
    let reader = BufReader::new(ref_list);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                let line: Vec<&str> = l.split(".").map(|a| a.trim()).collect();
                if !list.contains_key(line[1]) {
                    // not in our rankings, skip
                    println!("Note: Character {} is not in your ranking, skipping...", line[1]);
                    continue;
                }
                // set the rank in target
                list.entry(line[1].to_string()).and_modify(|a| (*a).1 = line[0].parse().unwrap());
            }
            Err(error) => {
                eprintln!("\nError reading... {}", error);
                break;
            }
        }
    }

    // delete character that isn't in the target list
    list.retain(|_, rank| (*rank).1 != 0);

    // recalculate the original ranks
    let mut find = 1;
    let mut replace = 1;
    let mut found = false;
    let mut was_found = true;

    while replace <= list.len() {
        found = false;
        for i in list.values_mut() {
            if i.0 == find {
                if !was_found {
                    i.0 = replace;
                    find -= 1;
                }
                replace += 1;
                found = true;
            }
        }
        was_found = found;
        find += 1;
    }

    list
}

// calculates the Weighted Rank Correlation
pub fn wrc_calc(list: &HashMap<String, (usize, usize)>)
-> f64 {

    let n = list.len();

    // this value affacts how aggresive the weighting is
    // a value of 1 turns this into regular Spearman's Rho
    let p = 2;

    // kappa
    let kappa = |p: u32| -> f64 {
        (1..n + 1).map(|i| i.pow(p)).sum::<usize>() as f64
    };

    // intermediate stuff
    let stuff = list.values()
        .map(|i| i.1 * (n + 1 - i.0).pow(p) + i.0 * (n + 1 - i.1).pow(p))
        .sum::<usize>() as f64;

    // result
    ( (n as f64 + 1.0) * kappa(p) - stuff )
    / ( 2.0 * kappa(p + 1) - (n as f64 + 1.0) * kappa(p) )
}

// Eye candy!
pub fn show_plot(list: &HashMap<String, (usize, usize)>) {

    // construct the rank list
    let mut rank_list: Vec<f64> = Vec::with_capacity(list.len());
    let mut counter = 1;
    while counter <= list.len() {
        for i in list.values() {
            if i.0 == counter {
                rank_list.push(i.1 as f64);
                counter += 1;
            }
        }
    }

    // make the plot
    let mut plot = Plot::default();
    plot.set_domain(Domain(0.0..list.len() as f64))
        .set_codomain(Domain(0.0..list.len() as f64))
        .set_title("Rankings comparison.")
        .set_x_label("Your ranks")
        .set_y_label("Target ranks")
        .set_size(Size::new(160,90))
        .add_plot(Box::new(plot::Bars::new(rank_list)));

    println!("{plot}");
}