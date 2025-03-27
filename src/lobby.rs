// Tohorank: lobby functions, list and stats

use colored::Colorize;
use crate::{INCLUSIVE, DEVIATION_BAR, Chara, Tags, stats};

// Show detailed stats about a character
pub fn stat(chara: &Chara, touhous: &Vec<Chara>, full_rankings: bool) {
    println!("{:-<1$}", "", 58);
    // Name and overall rank
    let no_tags: Vec<(Tags, bool)> = vec![];
    let everyone = stats::filter_group(no_tags.clone(), touhous);
    let ranking_overall = stats::rank_in_group(chara, &everyone);
    println!("{0: <53}{1: >13}",
        format!("~~ {} ~~", chara.name.bold()),
        format!("Rank #{}/{}", ranking_overall.0, ranking_overall.1)
    );
    println!("{:-<1$}", "", 58);

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
pub fn list(mut touhous: Vec<&Chara>, first: usize, name_filter: &str) {
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