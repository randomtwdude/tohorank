// Glicko-2 rating system

use crate::{chara, Chara, Match};
use std::f64::consts::PI;
use std::collections::HashMap;
use colored::Colorize;

// Updates all ratings, data::write_data() after use.
pub fn calc(touhous: &mut Vec<Chara>, records: &Vec<Match>) {
    println!("Tallying {} matches...", records.len());

    if records.len() > 0 { // avoid cluttering the history
        chara::update_history(touhous, records);
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

    // save old ratings of battled characters
    let mut old_ratings: HashMap<usize, f64> = HashMap::new();
    for th in qt_v.keys() {
        old_ratings.insert(*th, touhous[*th].rank.rate);
    }

    // transform to the glicko-2 scale
    for th in touhous.iter_mut() {
        glicko_two_scale(&mut th.rank.rate, &mut th.rank.devi);
    }

    for battle in records.iter() {
        // fetch numbers
        let r1 = touhous[battle.one].rank.rate;
        let r2 = touhous[battle.two].rank.rate;
        let rd1 = touhous[battle.one].rank.devi;
        let rd2 = touhous[battle.two].rank.devi;
        let (s1, s2) = if battle.res == 2.0 {
            (0.25, 0.25) // both sides lose, but not as much as when only one side loses
        } else {
            (battle.res, 1.0 - battle.res)
        };
        // update v1
        let v1_add: f64 = part_v(&r1, &r2, &rd2);
        if let Some(v1) = qt_v.get_mut(&battle.one) {
            *v1 += v1_add;
        }
        // update v2
        let v2_add: f64 = part_v(&r2, &r1, &rd1);
        if let Some(v2) = qt_v.get_mut(&battle.two) {
            *v2 += v2_add;
        }
        // update d1
        let d1_add: f64 = part_d(&r1, &r2, &rd2, &s1);
        if let Some(d1) = qt_d.get_mut(&battle.one) {
            *d1 += d1_add;
        }
        // update d2
        let d2_add: f64 = part_d(&r2, &r1, &rd1, &s2);
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
    const TAU: f64 = 0.5;

    // update the volatility for all characters in this session
    for th in qt_v.keys() {
        touhous[*th].rank.vola = calc_new_volatility(
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
        th.rank.devi = adjust_deviation(
            &th.rank.devi,
            &th.rank.vola
        );
    }
    // second round on battled characters
    for th in qt_v.keys() {
        touhous[*th].rank.devi = calc_new_deviation(
            &touhous[*th].rank.devi,
            &qt_v[th]
        );
    }

    // finally, we calculate the new ratings
    for th in qt_v.keys() {
        touhous[*th].rank.rate = calc_new_rating(
            &touhous[*th].rank.rate,
            &touhous[*th].rank.devi,
            &qt_v[th],
            &qt_d[th]
        );
    }

    // transform back to glicko scale
    for th in touhous.iter_mut() {
        glicko_one_scale(&mut th.rank.rate, &mut th.rank.devi);
    }

    // display changes
    println!("----- Changes -----");
    let mut diffs: Vec<(usize, f64)> = old_ratings
        .iter()
        .map(|(id, old_rt)| (*id, touhous[*id].rank.rate - old_rt))
        .collect();
    diffs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (th, diff) in diffs {
        println!("{:<26}| {:<5} | {}",
            touhous[th].name,
            format!("{:.0}", touhous[th].rank.rate),
            if diff > 0.0 {
                format!("{:.0}", diff).blue()
            } else {
                format!("{:.0}", diff).red()
            }
        );
    }
    println!("-------------------");
}

// convert from glicko to glicko-2
pub fn glicko_two_scale(rate: &mut f64, devi: &mut f64) {
    *rate = (*rate - 1500.0) / 173.7178;
    *devi /= 173.7178;
}

// convert back
pub fn glicko_one_scale(rate: &mut f64, devi: &mut f64) {
    *rate = *rate * 173.7178 + 1500.0;
    *devi *= 173.7178;
}

// For steps 3 and 4

// the function g from the paper
pub fn g(phi: &f64) -> f64 {
    1.0 / (1.0 + 3.0 * phi.powi(2) / PI.powi(2)).sqrt()
}

// the function E from the paper
pub fn e(mu: &f64, muj: &f64, phij: &f64) -> f64 {
    1.0 / (1.0 + (-1.0 * g(phij) * (mu - muj)).exp())
}

// one part of the v quantity
pub fn part_v(mu: &f64, muj: &f64, phij: &f64) -> f64 {
    let res_e = e(mu, muj, phij);
    g(phij).powi(2) * res_e * (1.0 - res_e)
}

// one part of the delta quantity
pub fn part_d(mu: &f64, muj: &f64, phij: &f64, res: &f32) -> f64 {
    g(phij) * (*res as f64 - e(mu, muj, phij))
}

// For step 5

// a = ln(sig^2)
pub fn a(sig: &f64) -> f64 {
    sig.powi(2).ln()
}

// f(x)
pub fn f(x: &f64, v: &f64, d: &f64, sig: &f64, phi: &f64, tau: &f64) -> f64 {
    (x.exp() * (d.powi(2) - phi.powi(2) - v - x.exp())) / (2.0 * (phi.powi(2) + v + x.exp()).powi(2)) -
    (x - a(sig)) / (tau.powi(2))
}

/* v: v-value
 * d: delta-value
 * sig: volatility
 * phi: deviation
 * tau: system constant
 * ep: convergence tolerance
 */
// holy that's a lot of variables
pub fn calc_new_volatility(v: &f64, d: &f64, sig: &f64, phi: &f64, tau: &f64, ep: &f64) -> f64 {
    // f(x) wrapper
    let ff = | x | {
        f(&x, &v, &d, &sig, &phi, &tau)
    };
    // 1. Set A = a = ln(sig^2)
    let (mut big_a, a) = (a(sig), a(sig));
    // 2. set the initial values of the algorithm
    let mut big_b = if d.powi(2) > phi.powi(2) + v {
        (d.powi(2) - phi.powi(2) - v).ln()
    } else {
        let mut k = 1.0;
        while ff(a - k * tau) < 0.0 {
            k += 1.0;
        }
        a - (k * tau)
    };
    // 3,4.
    let (mut fa, mut fb) = (ff(big_a), ff(big_b));
    while (big_b - big_a).abs() > *ep {
        let big_c = big_a + (big_a - big_b) * fa / (fb - fa);
        let fc = ff(big_c);
        if fc.is_sign_positive() != fb.is_sign_positive() || fc == 0.0 || fb == 0.0 {
            big_a = big_b;
            fa = fb;
        } else {
            fa /= 2.0;
        }
        big_b = big_c;
        fb = fc;
    }
    // 5.
    (big_a / 2.0).exp()
}

/* phi: old RD
 * sig: old/new volatility (depends on if they participated)
 */
// updates the deviation, this is needed for ALL characters every session
pub fn adjust_deviation(phi: &f64, sig: &f64) -> f64 {
    (phi.powi(2) + sig.powi(2)).sqrt()
}

/* phi: adjusted RD from adjust_deviation()
 * v: v-value
 */
// calculate new deviation, this is only need for characters that
// were involved in a session
pub fn calc_new_deviation(phi: &f64, v: &f64) -> f64 {
    1.0 / ((1.0 / phi.powi(2)) + (1.0 / v)).sqrt()
}

/* mu: old rating
 * phi: new RD
 * v, d: values
 */
// calculate new rating
pub fn calc_new_rating(mu: &f64, phi: &f64, v: &f64, d: &f64) -> f64 {
    mu + (d * phi.powi(2) / v)
}