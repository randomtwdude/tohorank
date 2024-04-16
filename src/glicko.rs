// All the math for the Glicko calculations

use std::f64::consts::PI;

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

// prints helpful info
pub fn glicko_info() {
    println!("About the Glicko rating system:");
    // TODO: add intrduction
}