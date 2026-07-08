//! Core calculator state machine.
//! Handles all 4 modes. All side-effects go through `handle_input`.

use crate::{
    history::History,
    memory::Memory,
    modes::{
        financial::{TvmState, margin_pct, tax_add, tax_remove},
        programmer::{Base, BitOp, ProgrammerState},
    },
    parser::fmt_num,
};

// ── Mode ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode { Basic=0, Scientific=1, Financial=2, Programmer=3 }

impl Mode {
    pub fn from_int(i: i32) -> Self {
        match i { 1=>Self::Scientific, 2=>Self::Financial, 3=>Self::Programmer, _=>Self::Basic }
    }
}

// ── Arithmetic op ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
enum Op { Add, Sub, Mul, Div, Mod, Pow }

impl Op {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "+"    => Some(Self::Add), "−"|"-" => Some(Self::Sub),
            "×"|"*"=> Some(Self::Mul), "÷"|"/" => Some(Self::Div),
            "%"    => Some(Self::Mod), "xʸ"|"^"=> Some(Self::Pow),
            _      => None,
        }
    }
    fn apply(self, a: f64, b: f64) -> Result<f64, String> {
        match self {
            Self::Add => Ok(a + b),
            Self::Sub => Ok(a - b),
            Self::Mul => Ok(a * b),
            Self::Div => { if b == 0.0 { Err("÷0".into()) } else { Ok(a / b) } }
            Self::Mod => Ok(a % b),
            Self::Pow => Ok(a.powf(b)),
        }
    }
    fn symbol(self) -> &'static str {
        match self { Self::Add=>"+", Self::Sub=>"−", Self::Mul=>"×",
                     Self::Div=>"÷", Self::Mod=>"%", Self::Pow=>"^" }
    }
}

// ── Financial CPT state ───────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum FinMode { Normal, Cpt }

// ═══════════════════════════════════════════════════════════════════════════
pub struct Calculator {
    pub mode:       Mode,
    pub display:    String,
    pub expression: String,
    pub memory:     Memory,
    pub history:    History,

    // Shared numeric state
    input:          String,
    pending_op:     Option<Op>,
    pending_val:    f64,
    should_reset:   bool,
    error:          bool,

    // Scientific
    pub degree_mode: bool,

    // Financial
    pub tvm:      TvmState,
    pub tvm_sel:  String,
    fin_mode:     FinMode,
    pub tax_rate: f64,

    // Programmer
    pub prog: ProgrammerState,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            mode:         Mode::Basic,
            display:      "0".into(),
            expression:   String::new(),
            memory:       Memory::new(),
            history:      History::new(),
            input:        "0".into(),
            pending_op:   None,
            pending_val:  0.0,
            should_reset: false,
            error:        false,
            degree_mode:  true,
            tvm:          TvmState::new(),
            tvm_sel:      String::new(),
            fin_mode:     FinMode::Normal,
            tax_rate:     20.0,
            prog:         ProgrammerState::new(),
        }
    }

    pub fn set_mode(&mut self, m: i32) {
        self.mode = Mode::from_int(m);
        self.reset_arith();
        self.fin_mode = FinMode::Normal;
        self.sync_display();
    }

    // ── Main dispatch ─────────────────────────────────────────────────────
    pub fn handle_input(&mut self, btn: &str) {
        match self.mode {
            Mode::Basic      => self.basic(btn),
            Mode::Scientific => self.scientific(btn),
            Mode::Financial  => self.financial(btn),
            Mode::Programmer => self.programmer(btn),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BASIC
    // ═══════════════════════════════════════════════════════════════════════
    fn basic(&mut self, btn: &str) {
        match btn {
            "AC"  => self.clear_all(),
            "⌫"  => self.backspace(),
            "="   => self.evaluate_pending(),
            "%"   => self.percent_op(),
            "MC"  => { self.memory.clear(); }
            "MR"  => { let v = self.memory.recall(); self.set_num(v); }
            "M+"  => { self.memory.add(self.cur_f64()); }
            "M-"  => { self.memory.sub(self.cur_f64()); }
            "MS"  => { self.memory.store(self.cur_f64()); }
            d if is_digit_btn(d) => self.append_digit(d),
            op    if Op::from_str(op).is_some() => self.choose_op(op),
            _ => {}
        }
        self.sync_display();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SCIENTIFIC
    // ═══════════════════════════════════════════════════════════════════════
    fn scientific(&mut self, btn: &str) {
        // Shared controls
        match btn {
            "AC" => { self.clear_all(); return; }
            "⌫"  => { self.backspace(); self.sync_display(); return; }
            "="   => { self.evaluate_pending(); return; }
            "%"   => { self.percent_op(); self.sync_display(); return; }
            "DEG/RAD" => { self.degree_mode = !self.degree_mode; self.sync_display(); return; }
            _ => {}
        }

        // Unary functions applied to current display value
        let v = self.cur_f64();
        let deg = self.degree_mode;
        let to_rad   = if deg { std::f64::consts::PI / 180.0 } else { 1.0 };
        let from_rad = if deg { 180.0 / std::f64::consts::PI } else { 1.0 };

        let result: Option<Result<f64, String>> = match btn {
            "sin"  => Some(Ok((v * to_rad).sin())),
            "cos"  => Some(Ok((v * to_rad).cos())),
            "tan"  => Some(Ok((v * to_rad).tan())),
            "asin" => Some(if v.abs()>1.0 {Err("Domain".into())} else {Ok(v.asin()*from_rad)}),
            "acos" => Some(if v.abs()>1.0 {Err("Domain".into())} else {Ok(v.acos()*from_rad)}),
            "atan" => Some(Ok(v.atan() * from_rad)),
            "log"  => Some(if v<=0.0 {Err("Domain".into())} else {Ok(v.log10())}),
            "ln"   => Some(if v<=0.0 {Err("Domain".into())} else {Ok(v.ln())}),
            "√x"   => Some(if v<0.0  {Err("√ neg".into())}  else {Ok(v.sqrt())}),
            "1/x"  => Some(if v==0.0 {Err("1/0".into())}    else {Ok(1.0/v)}),
            "x!"   => Some({
                if v<0.0||!is_int_f64(v)||v>170.0 {Err("Domain".into())}
                else {Ok(factorial(v as u64) as f64)}
            }),
            "RND"  => Some(Ok(pseudo_rand())),
            "π"    => Some(Ok(std::f64::consts::PI)),
            "e"    => Some(Ok(std::f64::consts::E)),
            _      => None,
        };

        if let Some(res) = result {
            match res {
                Ok(r)  => { self.set_num(r); }
                Err(e) => { self.set_error(&e); }
            }
            self.sync_display();
            return;
        }

        // xʸ becomes pending power op
        if btn == "xʸ" { self.choose_op("xʸ"); self.sync_display(); return; }

        // Digits and remaining ops
        if is_digit_btn(btn)          { self.append_digit(btn); }
        else if Op::from_str(btn).is_some() { self.choose_op(btn); }
        self.sync_display();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FINANCIAL
    // ═══════════════════════════════════════════════════════════════════════
    fn financial(&mut self, btn: &str) {
        match btn {
            "AC" => {
                self.clear_all();
                self.fin_mode = FinMode::Normal;
                self.tvm_sel  = String::new();
                self.sync_display();
                return;
            }
            "⌫" => { self.backspace(); self.sync_display(); return; }
            "CLR TVM" => {
                self.tvm.clear();
                self.fin_mode = FinMode::Normal;
                self.tvm_sel  = String::new();
                self.clear_all();
                return;
            }
            "CPT" => {
                self.fin_mode = FinMode::Cpt;
                self.expression = "CPT →".into();
                self.sync_display();
                return;
            }
            _ => {}
        }

        // TVM slot buttons
        if let Some(slot) = btn.strip_prefix("TVM:") {
            let slot = slot.to_string();
            if self.fin_mode == FinMode::Cpt {
                // Compute the missing variable
                match self.tvm.compute(&slot) {
                    Ok(v) => {
                        self.store_tvm(&slot, v);
                        let expr = format!("{slot} = {}", fmt_num(v));
                        self.history.push(&expr, fmt_num(v));
                        self.expression = expr;
                        self.set_num(v);
                    }
                    Err(e) => self.set_error(&e),
                }
                self.fin_mode = FinMode::Normal;
                self.tvm_sel  = String::new();
            } else {
                // Store current display into TVM slot
                let v = self.cur_f64();
                self.store_tvm(&slot, v);
                self.expression = format!("{slot} = {}", fmt_num(v));
                self.tvm_sel    = slot;
                self.should_reset = true;
            }
            self.sync_display();
            return;
        }

        // Tax / Margin helpers
        match btn {
            "Tax+" => {
                let base  = self.cur_f64();
                let total = tax_add(base, self.tax_rate);
                self.expression = format!("{} + {}% = {}", fmt_num(base), self.tax_rate, fmt_num(total));
                self.set_num(total);
                self.sync_display();
                return;
            }
            "Tax−" => {
                let total = self.cur_f64();
                let base  = tax_remove(total, self.tax_rate);
                self.expression = format!("{} (incl {}% tax, net = {})", fmt_num(total), self.tax_rate, fmt_num(base));
                self.set_num(base);
                self.sync_display();
                return;
            }
            "Mar" => {
                let sell = self.cur_f64();
                let cost = self.tvm.pv.unwrap_or(self.pending_val);
                match margin_pct(cost, sell) {
                    Ok(pct) => {
                        self.expression = format!("Margin {}/{} = {:.2}%", fmt_num(cost), fmt_num(sell), pct);
                        self.set_num(pct);
                    }
                    Err(e) => self.set_error(&e),
                }
                self.sync_display();
                return;
            }
            _ => {}
        }

        // Standard arithmetic
        match btn {
            "=" => self.evaluate_pending(),
            "%" => self.percent_op(),
            d if is_digit_btn(d)           => self.append_digit(d),
            op if Op::from_str(op).is_some()=> self.choose_op(op),
            _ => {}
        }
        self.sync_display();
    }

    fn store_tvm(&mut self, slot: &str, v: f64) {
        match slot {
            "N"   => self.tvm.n   = Some(v),
            "I/Y" => self.tvm.iy  = Some(v),
            "PV"  => self.tvm.pv  = Some(v),
            "PMT" => self.tvm.pmt = Some(v),
            "FV"  => self.tvm.fv  = Some(v),
            _ => {}
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PROGRAMMER
    // ═══════════════════════════════════════════════════════════════════════
    fn programmer(&mut self, btn: &str) {
        match btn {
            "AC" => { self.prog.clear(); }
            "⌫"  => { self.prog.backspace(); }
            "="   => { let _v = self.prog.compute(); }
            "NOT" => { self.prog.not(); }
            "AND" => { self.prog.set_op(BitOp::And); }
            "OR"  => { self.prog.set_op(BitOp::Or);  }
            "XOR" => { self.prog.set_op(BitOp::Xor); }
            "<<"  => { self.prog.set_op(BitOp::Shl); }
            ">>"  => { self.prog.set_op(BitOp::Shr); }
            s if s.starts_with("BASE:") => {
                if let Some(b) = Base::from_name(&s[5..]) { self.prog.set_base(b); }
            }
            ch if ch.len() == 1 => {
                if let Some(c) = ch.chars().next() { self.prog.digit(c); }
            }
            _ => {}
        }
        self.display    = self.prog.current_display();
        self.expression = String::new();
    }

    // ── Programmer display helpers (called from app.rs) ───────────────────
    pub fn hex_text(&self) -> String { self.prog.display_in(Base::Hex) }
    pub fn dec_text(&self) -> String { self.prog.display_in(Base::Dec) }
    pub fn oct_text(&self) -> String { self.prog.display_in(Base::Oct) }
    pub fn bin_text(&self) -> String { self.prog.display_in(Base::Bin) }

    // ═══════════════════════════════════════════════════════════════════════
    // Shared arithmetic helpers
    // ═══════════════════════════════════════════════════════════════════════

    fn append_digit(&mut self, d: &str) {
        if self.error { self.clear_all(); }
        if self.should_reset { self.input = String::new(); self.should_reset = false; }
        match d {
            "." => {
                if !self.input.contains('.') {
                    if self.input.is_empty() { self.input.push('0'); }
                    self.input.push('.');
                }
            }
            _ => {
                if self.input == "0"       { self.input  = d.into(); }
                else if self.input.len()<16 { self.input.push_str(d); }
            }
        }
    }

    fn choose_op(&mut self, op: &str) {
        if self.error { return; }
        // Chain: if pending op, evaluate first
        if self.pending_op.is_some() && !self.should_reset {
            if let Some(o) = self.pending_op {
                if let Ok(r) = o.apply(self.pending_val, self.cur_f64()) {
                    self.pending_val = r;
                    self.input       = fmt_num(r);
                }
            }
        }
        self.pending_val  = self.cur_f64();
        self.pending_op   = Op::from_str(op);
        self.expression   = format!("{} {}", fmt_num(self.pending_val), op);
        self.should_reset = true;
    }

    fn evaluate_pending(&mut self) {
        if self.error  { self.clear_all(); return; }
        if self.pending_op.is_none() { return; }

        let lhs   = self.pending_val;
        let rhs   = self.cur_f64();
        let sym   = self.pending_op.map(|o| o.symbol()).unwrap_or("");
        let expr  = format!("{} {sym} {} =", fmt_num(lhs), fmt_num(rhs));

        match self.pending_op.unwrap().apply(lhs, rhs) {
            Ok(r) => {
                let s = fmt_num(r);
                self.history.push(&expr, &s);
                self.expression   = expr;
                self.input        = s.clone();
                self.display      = s;
                self.pending_op   = None;
                self.pending_val  = 0.0;
                self.should_reset = true;
            }
            Err(e) => self.set_error(&e),
        }
    }

    fn percent_op(&mut self) {
        let v = self.cur_f64();
        let result = match self.pending_op {
            Some(Op::Add) | Some(Op::Sub) => self.pending_val * v / 100.0,
            _ => v / 100.0,
        };
        self.input        = fmt_num(result);
        self.should_reset = false;
    }

    fn cur_f64(&self) -> f64 { self.input.parse().unwrap_or(0.0) }

    fn set_num(&mut self, v: f64) {
        self.input        = fmt_num(v);
        self.display      = self.input.clone();
        self.should_reset = true;
    }

    fn set_error(&mut self, msg: &str) {
        self.display      = "Error".into();
        self.expression   = msg.into();
        self.error        = true;
        self.input        = "0".into();
        self.should_reset = true;
    }

    fn clear_all(&mut self) {
        self.input        = "0".into();
        self.pending_op   = None;
        self.pending_val  = 0.0;
        self.should_reset = false;
        self.error        = false;
        self.display      = "0".into();
        self.expression   = String::new();
    }

    fn backspace(&mut self) {
        if self.error        { self.clear_all(); return; }
        if self.should_reset { return; }
        if self.input.len() > 1 { self.input.pop(); }
        else { self.input = "0".into(); }
    }

    fn reset_arith(&mut self) {
        self.input        = "0".into();
        self.pending_op   = None;
        self.pending_val  = 0.0;
        self.should_reset = false;
        self.error        = false;
        self.display      = "0".into();
        self.expression   = String::new();
    }

    fn sync_display(&mut self) {
        if !self.error { self.display = self.input.clone(); }
    }
}

impl Default for Calculator { fn default() -> Self { Self::new() } }

// ── Utilities ─────────────────────────────────────────────────────────────
fn is_digit_btn(s: &str) -> bool {
    matches!(s, "0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9"|".")
    || (s.len()==1 && s.chars().next().map(|c| c.is_ascii_hexdigit()).unwrap_or(false))
}

fn is_int_f64(v: f64) -> bool { (v - v.floor()).abs() < 1e-9 }

fn factorial(n: u64) -> u64 { (1..=n).fold(1u64, |a,i| a.saturating_mul(i)) }

fn pseudo_rand() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345);
    let x = ns.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (x as f64 / u32::MAX as f64).abs().fract()
}
