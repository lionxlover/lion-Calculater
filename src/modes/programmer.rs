#![allow(dead_code)]
//! Programmer calculator engine — integer arithmetic, base conversion, bit ops.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base { Hex = 16, Dec = 10, Oct = 8, Bin = 2 }

impl Base {
    pub fn name(self) -> &'static str {
        match self { Self::Hex=>"HEX", Self::Dec=>"DEC", Self::Oct=>"OCT", Self::Bin=>"BIN" }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s { "HEX"=>Some(Self::Hex), "DEC"=>Some(Self::Dec),
                  "OCT"=>Some(Self::Oct), "BIN"=>Some(Self::Bin), _=>None }
    }
    pub fn radix(self) -> u32 { self as u32 }
    pub fn digit_ok(self, c: char) -> bool {
        match self {
            Self::Bin => matches!(c, '0'|'1'),
            Self::Oct => matches!(c, '0'..='7'),
            Self::Dec => c.is_ascii_digit(),
            Self::Hex => c.is_ascii_hexdigit(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitOp { And, Or, Xor, Shl, Shr }

#[derive(Debug)]
struct Pending { lhs: i64, op: BitOp }

#[derive(Debug)]
pub struct ProgrammerState {
    pub base:      Base,
    pub bit_width: u8,   // 8 | 16 | 32 | 64
    input:         String,
    pending:       Option<Pending>,
}

impl ProgrammerState {
    pub fn new() -> Self {
        Self { base: Base::Dec, bit_width: 64, input: "0".into(), pending: None }
    }

    // ── Value ─────────────────────────────────────────────────────────────
    fn raw_value(&self) -> i64 {
        i64::from_str_radix(&self.input, self.base.radix()).unwrap_or(0)
    }
    fn mask(&self, v: i64) -> i64 {
        match self.bit_width { 8=>v as i8 as i64, 16=>v as i16 as i64,
                               32=>v as i32 as i64, _=>v }
    }
    pub fn value(&self) -> i64 { self.mask(self.raw_value()) }

    pub fn display_in(&self, b: Base) -> String {
        let v = self.value();
        match b {
            Base::Dec => format!("{v}"),
            Base::Hex => format!("{:X}", v as u64),
            Base::Oct => format!("{:o}", v as u64),
            Base::Bin => format!("{:b}", v as u64),
        }
    }

    pub fn current_display(&self) -> String { self.input.clone() }

    // ── Input ─────────────────────────────────────────────────────────────
    pub fn clear(&mut self) { self.input = "0".into(); self.pending = None; }

    pub fn backspace(&mut self) {
        if self.input.len() > 1 { self.input.pop(); }
        else { self.input = "0".into(); }
    }

    pub fn digit(&mut self, c: char) {
        if !self.base.digit_ok(c) { return; }
        let ch = c.to_ascii_uppercase().to_string();
        if self.input == "0" { self.input = ch; }
        else if self.input.len() < 20 { self.input.push_str(&ch); }
    }

    // ── Operations ────────────────────────────────────────────────────────
    pub fn set_op(&mut self, op: BitOp) {
        self.pending = Some(Pending { lhs: self.value(), op });
        self.input   = "0".into();
    }

    pub fn compute(&mut self) -> i64 {
        if let Some(p) = self.pending.take() {
            let rhs = self.value();
            let res = match p.op {
                BitOp::And => p.lhs & rhs,
                BitOp::Or  => p.lhs | rhs,
                BitOp::Xor => p.lhs ^ rhs,
                BitOp::Shl => p.lhs.wrapping_shl(rhs.unsigned_abs() as u32),
                BitOp::Shr => p.lhs.wrapping_shr(rhs.unsigned_abs() as u32),
            };
            let res = self.mask(res);
            self.set_value(res);
        }
        self.value()
    }

    pub fn not(&mut self) { let v = self.mask(!self.value()); self.set_value(v); }

    fn set_value(&mut self, v: i64) {
        self.input = match self.base {
            Base::Dec => format!("{v}"),
            Base::Hex => format!("{:X}", v as u64),
            Base::Oct => format!("{:o}", v as u64),
            Base::Bin => format!("{:b}", v as u64),
        };
        if self.input.is_empty() { self.input = "0".into(); }
    }

    pub fn set_base(&mut self, b: Base) {
        let v     = self.value();
        self.base = b;
        self.set_value(v);
    }

    pub fn load_f64(&mut self, v: f64) { self.set_value(v as i64); }
}

impl Default for ProgrammerState { fn default() -> Self { Self::new() } }
