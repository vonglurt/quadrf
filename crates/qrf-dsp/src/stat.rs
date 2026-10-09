//! CA-CFAR thresholds (analysis T25; the same routines as
//! `analysis/linkbudget.py`): with the noise mean known, a power averaged
//! over k exponential looks exceeds α·mean with probability Q(k, k·α), Q the
//! regularised upper incomplete gamma function; with the noise mean taken
//! from training cells whose average is Gamma-distributed with shape K, the
//! ratio law P_fa = I_{K/(K + αk)}(K, k) applies, I the regularised
//! incomplete beta function.

/// Regularised upper incomplete gamma Q(a, x) for integer a (series and continued fraction); ln Γ(a) as a sum of logs.
pub fn gamma_q(a: u32, x: f64) -> f64 {
    let af = f64::from(a);
    let mut lg = 0.0;
    for i in 1..a {
        lg += f64::from(i).ln();
    }
    let pre = (-x + af * x.ln() - lg).exp();
    if x < af + 1.0 {
        let (mut ap, mut s, mut d) = (af, 1.0 / af, 1.0 / af);
        for _ in 0..500 {
            ap += 1.0;
            d *= x / ap;
            s += d;
            if d.abs() < s.abs() * 1e-15 {
                break;
            }
        }
        return 1.0 - s * pre;
    }
    let mut b = x + 1.0 - af;
    let mut c = 1.0 / 1e-300;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..500 {
        let an = -(i as f64) * (i as f64 - af);
        b += 2.0;
        d = an * d + b;
        if d.abs() < 1e-300 {
            d = 1e-300;
        }
        c = b + an / c;
        if c.abs() < 1e-300 {
            c = 1e-300;
        }
        d = 1.0 / d;
        let de = d * c;
        h *= de;
        if (de - 1.0).abs() < 1e-15 {
            break;
        }
    }
    pre * h
}

/// Threshold over the (exactly known) noise mean such that a power averaged over `k` exponential looks exceeds it with probability `pfa`.
pub fn cfar_alpha(k: u32, pfa: f64) -> f64 {
    let (mut lo, mut hi) = (1.0f64, 1000.0f64);
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if gamma_q(k, f64::from(k) * mid) > pfa {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// ln Γ(n) = ln((n−1)!) for integer n ≥ 1, from a cumulative table of logs.
pub fn lg_int(n: usize) -> f64 {
    thread_local! { static LG: std::cell::RefCell<Vec<f64>> = std::cell::RefCell::new(vec![0.0, 0.0]); }
    LG.with(|t| {
        let mut t = t.borrow_mut();
        while t.len() <= n {
            let v = t[t.len() - 1] + ((t.len() - 1) as f64).ln();
            t.push(v);
        }
        t[n]
    })
}

/// Continued fraction for the incomplete beta function (Numerical Recipes).
fn betacf(a: f64, b: f64, x: f64) -> f64 {
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < 1e-300 {
        d = 1e-300;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=1000 {
        let mf = m as f64;
        let m2 = 2.0 * mf;
        let mut aa = mf * (b - mf) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-300 {
            d = 1e-300;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-300 {
            c = 1e-300;
        }
        d = 1.0 / d;
        h *= d * c;
        aa = -(a + mf) * (qab + mf) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-300 {
            d = 1e-300;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-300 {
            c = 1e-300;
        }
        d = 1.0 / d;
        let de = d * c;
        h *= de;
        if (de - 1.0).abs() < 3e-16 {
            break;
        }
    }
    h
}

/// Regularised incomplete beta I_x(a, b) for integer a, b.
pub fn incbeta(a: usize, b: usize, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let (af, bf) = (a as f64, b as f64);
    let bt = (lg_int(a + b) - lg_int(a) - lg_int(b) + af * x.ln() + bf * (1.0 - x).ln()).exp();
    if x < (af + 1.0) / (af + bf + 2.0) { bt * betacf(af, bf, x) / af } else { 1.0 - bt * betacf(bf, af, 1.0 - x) / bf }
}

/// CA-CFAR threshold over the training mean: the cell under test averages `k` looks, the training mean has Gamma shape `kk` (N·k for N equal cells); P_fa = I_{K/(K + αk)}(K, k).
pub fn cfar_alpha_ca(k: usize, kk: usize, pfa: f64) -> f64 {
    let (mut lo, mut hi) = (1.0f64, 1000.0f64);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if incbeta(kk, k, kk as f64 / (kk as f64 + mid * k as f64)) > pfa {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_law_matches_the_closed_form_and_t25() {
        // T25: 16 single-look cells, Pfa 1e-3: 8.639 both ways
        assert!((cfar_alpha_ca(1, 16, 1e-3) - 8.639).abs() < 5e-4);
        // T25: k=197, K=3588 -> 1.243; k=389, K=1794 -> 1.184
        assert!((cfar_alpha_ca(197, 3588, 1e-3) - 1.243).abs() < 5e-4);
        assert!((cfar_alpha_ca(389, 1794, 1e-3) - 1.184).abs() < 5e-4);
        // K -> large approaches the gamma tail
        assert!((cfar_alpha_ca(203, 2_000_000, 1e-3) - cfar_alpha(203, 1e-3)).abs() < 2e-3);
        assert!((incbeta(2, 3, 0.5) - 0.6875).abs() < 1e-12);
    }

    #[test]
    fn matches_t25() {
        // T25: k=1 -> -ln(1e-3) = 6.908; k=203 -> 1.231; k=406 -> 1.160 (Pfa 1e-3)
        assert!((cfar_alpha(1, 1e-3) - 6.908).abs() < 5e-4);
        assert!((cfar_alpha(203, 1e-3) - 1.231).abs() < 5e-4);
        assert!((cfar_alpha(406, 1e-3) - 1.160).abs() < 5e-4);
        assert!((cfar_alpha(32, 1e-4) - 1.794).abs() < 5e-4);
    }

    #[test]
    fn gamma_q_limits() {
        assert!((gamma_q(1, 0.0) - 1.0).abs() < 1e-12);
        assert!((gamma_q(1, 1.0) - (-1.0f64).exp()).abs() < 1e-12);
        assert!((gamma_q(3, 2.0) - 0.676_676_416_183_063).abs() < 1e-10);
    }
}
