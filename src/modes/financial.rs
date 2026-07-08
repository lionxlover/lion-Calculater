//! Time-Value-of-Money solver and basic financial utilities.
//!
//! Standard TVM equation (end-of-period payments, ordinary annuity):
//!   PV·(1+i)^N  +  PMT·[(1+i)^N − 1]/i  +  FV  =  0
//!
//! where  i = I/Y / 100   (periodic rate already expressed as a decimal)

#[derive(Debug, Clone, Default)]
pub struct TvmState {
    pub n:   Option<f64>,
    pub iy:  Option<f64>,
    pub pv:  Option<f64>,
    pub pmt: Option<f64>,
    pub fv:  Option<f64>,
}

impl TvmState {
    pub fn new() -> Self { Self::default() }
    pub fn clear(&mut self) { *self = Self::default(); }

    // Display strings ---------------------------------------------------------
    pub fn n_str(&self)   -> String { fmt(self.n)   }
    pub fn iy_str(&self)  -> String { fmt(self.iy)  }
    pub fn pv_str(&self)  -> String { fmt(self.pv)  }
    pub fn pmt_str(&self) -> String { fmt(self.pmt) }
    pub fn fv_str(&self)  -> String { fmt(self.fv)  }

    // Solver ------------------------------------------------------------------
    pub fn compute(&self, target: &str) -> Result<f64, String> {
        match target {
            "FV"  => self.solve_fv(),
            "PV"  => self.solve_pv(),
            "PMT" => self.solve_pmt(),
            "N"   => self.solve_n(),
            "I/Y" => self.solve_iy(),
            _     => Err(format!("Unknown TVM target: {target}")),
        }
    }

    // FV = -(PV·(1+i)^N + PMT·((1+i)^N-1)/i)
    fn solve_fv(&self) -> Result<f64, String> {
        let (n, i, pv, pmt) = self.need("FV", &["N","I/Y","PV","PMT"])?;
        let r = i / 100.0;
        let fv = if r == 0.0 {
            -(pv + pmt * n)
        } else {
            let f = (1.0 + r).powf(n);
            -(pv * f + pmt * (f - 1.0) / r)
        };
        Ok(r12(fv))
    }

    // PV = -(FV + PMT·((1+i)^N-1)/i) / (1+i)^N
    fn solve_pv(&self) -> Result<f64, String> {
        let (n, i, pmt, fv) = self.need("PV", &["N","I/Y","PMT","FV"])?;
        let r = i / 100.0;
        let pv = if r == 0.0 {
            -(fv + pmt * n)
        } else {
            let f = (1.0 + r).powf(n);
            -(fv + pmt * (f - 1.0) / r) / f
        };
        Ok(r12(pv))
    }

    // PMT = -r·(PV·(1+i)^N + FV) / ((1+i)^N - 1)
    fn solve_pmt(&self) -> Result<f64, String> {
        let (n, i, pv, fv) = self.need("PMT", &["N","I/Y","PV","FV"])?;
        let r = i / 100.0;
        let pmt = if r == 0.0 {
            if n == 0.0 { return Err("N cannot be zero".into()); }
            -(pv + fv) / n
        } else {
            let f = (1.0 + r).powf(n);
            -r * (pv * f + fv) / (f - 1.0)
        };
        Ok(r12(pmt))
    }

    fn solve_n(&self) -> Result<f64, String> {
        let (i, pv, pmt, fv) = self.need("N", &["I/Y","PV","PMT","FV"])?;
        let r = i / 100.0;
        let n = if r == 0.0 {
            if pmt == 0.0 { return Err("PMT cannot be zero when I/Y = 0".into()); }
            -(pv + fv) / pmt
        } else {
            let a = pmt - fv * r;
            let b = pmt + pv * r;
            if a <= 0.0 || b <= 0.0 {
                return Err("Cannot solve N: check signs of PV / FV / PMT".into());
            }
            (a / b).ln() / (1.0 + r).ln()
        };
        Ok(r12(n))
    }

    // Newton-Raphson for I/Y (transcendental)
    fn solve_iy(&self) -> Result<f64, String> {
        let (n, pv, pmt, fv) = self.need("I/Y", &["N","PV","PMT","FV"])?;

        // Special case: zero PMT
        if pmt.abs() < 1e-14 {
            if pv == 0.0 { return Err("PV cannot be zero".into()); }
            let ratio = -fv / pv;
            if ratio <= 0.0 { return Err("Cannot solve I/Y: negative ratio".into()); }
            return Ok(r12((ratio.powf(1.0 / n) - 1.0) * 100.0));
        }

        let f  = |r: f64| -> f64 {
            let factor = (1.0 + r).powf(n);
            pv * factor + pmt * (factor - 1.0) / r + fv
        };
        let df = |r: f64| -> f64 {
            let factor = (1.0 + r).powf(n);
            n * pv * factor / (1.0 + r)
                + pmt * (n * factor / (1.0 + r) * r - (factor - 1.0)) / (r * r)
        };

        let mut r = 0.1_f64;
        for _ in 0..300 {
            let fr  = f(r);
            let dfr = df(r);
            if dfr.abs() < 1e-15 { break; }
            let r2 = r - fr / dfr;
            if (r2 - r).abs() < 1e-12 { return Ok(r12(r2 * 100.0)); }
            r = r2.clamp(-0.9999, 100.0);
        }
        Err("I/Y solver did not converge — check inputs".into())
    }

    // Helper: extract exactly 4 named TVM values
    fn need(&self, solving: &str, names: &[&str; 4]) -> Result<(f64,f64,f64,f64), String> {
        let keys = ["N","I/Y","PV","PMT","FV"];
        let vals = [self.n, self.iy, self.pv, self.pmt, self.fv];
        let mut out = [0f64; 4];
        for (j, &name) in names.iter().enumerate() {
            let idx = keys.iter().position(|&k| k == name)
                .ok_or_else(|| format!("Unknown key '{name}'"))?;
            out[j] = vals[idx].ok_or_else(||
                format!("{name} is not set (needed to solve {solving})"))?;
        }
        Ok((out[0], out[1], out[2], out[3]))
    }
}

fn fmt(v: Option<f64>) -> String {
    v.map(|n| crate::parser::fmt_num(n)).unwrap_or_else(|| "?".into())
}

fn r12(v: f64) -> f64 { (v * 1e10).round() / 1e10 }

// ── Business helpers ──────────────────────────────────────────────────────

/// Add tax:    total = base × (1 + rate/100)
pub fn tax_add(base: f64, rate: f64) -> f64 { r12(base * (1.0 + rate / 100.0)) }

/// Remove tax: base  = total / (1 + rate/100)
pub fn tax_remove(total: f64, rate: f64) -> f64 { r12(total / (1.0 + rate / 100.0)) }

/// Gross margin %: (price − cost) / price × 100
pub fn margin_pct(cost: f64, price: f64) -> Result<f64, String> {
    if price == 0.0 { return Err("Price cannot be zero".into()); }
    Ok(r12((price - cost) / price * 100.0))
}
