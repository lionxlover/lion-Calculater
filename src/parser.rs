#![allow(dead_code)]
//! Recursive-descent expression parser / evaluator.
//!
//! Grammar
//! -------
//!   expr    → term   ( ('+' | '-')  term   )*
//!   term    → power  ( ('*' | '/' | '%') power )*
//!   power   → unary  ( '^' unary )*      right-associative
//!   unary   → '-'? primary
//!   primary → NUMBER | '(' expr ')' | IDENT | IDENT '(' expr ')'

use std::f64::consts::{E, PI};

// ── Tokens ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Plus, Minus, Star, Slash, Percent, Caret,
    LParen, RParen,
    Ident(String),
    End,
}

// ── Tokeniser ─────────────────────────────────────────────────────────────
fn tokenise(input: &str) -> Result<Vec<Tok>, String> {
    let mut toks  = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => { chars.next(); }

            '0'..='9' | '.' => {
                let mut s = String::new();
                let mut last_was_e = false;
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        s.push(c); chars.next(); last_was_e = false;
                    } else if c == 'e' || c == 'E' {
                        s.push(c); chars.next(); last_was_e = true;
                    } else if last_was_e && (c == '+' || c == '-') {
                        s.push(c); chars.next(); last_was_e = false;
                    } else { break; }
                }
                toks.push(Tok::Num(s.parse().map_err(|_| format!("Bad number: {s}"))?));
            }

            'a'..='z' | 'A'..='Z' | '_' | 'π' => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == 'π' {
                        s.push(c); chars.next();
                    } else { break; }
                }
                toks.push(Tok::Ident(s));
            }

            '+' => { toks.push(Tok::Plus);    chars.next(); }
            '-' => { toks.push(Tok::Minus);   chars.next(); }
            '*' | '×' => { toks.push(Tok::Star);    chars.next(); }
            '/' | '÷' => { toks.push(Tok::Slash);   chars.next(); }
            '%' => { toks.push(Tok::Percent); chars.next(); }
            '^' => { toks.push(Tok::Caret);   chars.next(); }
            '(' => { toks.push(Tok::LParen);  chars.next(); }
            ')' => { toks.push(Tok::RParen);  chars.next(); }
            c   => return Err(format!("Unknown character '{c}'")),
        }
    }
    toks.push(Tok::End);
    Ok(toks)
}

// ── Parser ────────────────────────────────────────────────────────────────
struct Parser<'a> { toks: &'a [Tok], pos: usize, deg: bool }

impl<'a> Parser<'a> {
    fn peek(&self) -> &Tok { &self.toks[self.pos] }
    fn bump(&mut self) -> &Tok {
        let t = &self.toks[self.pos];
        if self.pos + 1 < self.toks.len() { self.pos += 1; }
        t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == t { self.bump(); true } else { false }
    }
}

fn parse_expr(p: &mut Parser<'_>) -> Result<f64, String> {
    let mut v = parse_term(p)?;
    loop {
        match p.peek() {
            Tok::Plus  => { p.bump(); v += parse_term(p)?; }
            Tok::Minus => { p.bump(); v -= parse_term(p)?; }
            _          => break,
        }
    }
    Ok(v)
}

fn parse_term(p: &mut Parser<'_>) -> Result<f64, String> {
    let mut v = parse_power(p)?;
    loop {
        match p.peek() {
            Tok::Star    => { p.bump(); v *= parse_power(p)?; }
            Tok::Slash   => {
                p.bump();
                let d = parse_power(p)?;
                if d == 0.0 { return Err("Division by zero".into()); }
                v /= d;
            }
            Tok::Percent => { p.bump(); v %= parse_power(p)?; }
            _            => break,
        }
    }
    Ok(v)
}

fn parse_power(p: &mut Parser<'_>) -> Result<f64, String> {
    let base = parse_unary(p)?;
    if let Tok::Caret = p.peek() {
        p.bump();
        Ok(base.powf(parse_power(p)?))  // right-associative
    } else { Ok(base) }
}

fn parse_unary(p: &mut Parser<'_>) -> Result<f64, String> {
    if matches!(p.peek(), Tok::Minus) { p.bump(); Ok(-parse_primary(p)?) }
    else { parse_primary(p) }
}

fn parse_primary(p: &mut Parser<'_>) -> Result<f64, String> {
    match p.peek().clone() {
        Tok::Num(n) => { p.bump(); Ok(n) }

        Tok::LParen => {
            p.bump();
            let v = parse_expr(p)?;
            if !p.eat(&Tok::RParen) { return Err("Expected ')'".into()); }
            Ok(v)
        }

        Tok::Ident(name) => {
            p.bump();
            // Constants
            match name.as_str() {
                "pi" | "π" => return Ok(PI),
                "e"        => return Ok(E),
                _          => {}
            }
            // Functions
            if p.eat(&Tok::LParen) {
                let arg = parse_expr(p)?;
                if !p.eat(&Tok::RParen) {
                    return Err(format!("Expected ')' after {name}(...)"));
                }
                apply_fn(&name, arg, p.deg)
            } else {
                Err(format!("Unknown identifier '{name}'"))
            }
        }

        other => Err(format!("Unexpected token: {other:?}")),
    }
}

fn apply_fn(name: &str, x: f64, deg: bool) -> Result<f64, String> {
    let to_rad   = if deg { PI / 180.0 } else { 1.0 };
    let from_rad = if deg { 180.0 / PI } else { 1.0 };
    match name {
        "sin"   => Ok((x * to_rad).sin()),
        "cos"   => Ok((x * to_rad).cos()),
        "tan"   => Ok((x * to_rad).tan()),
        "asin"  => { if x.abs() > 1.0 { Err("asin: domain error".into()) }
                     else { Ok(x.asin() * from_rad) } }
        "acos"  => { if x.abs() > 1.0 { Err("acos: domain error".into()) }
                     else { Ok(x.acos() * from_rad) } }
        "atan"  => Ok(x.atan() * from_rad),
        "log"   => { if x <= 0.0 { Err("log: domain error".into()) }
                     else { Ok(x.log10()) } }
        "ln"    => { if x <= 0.0 { Err("ln: domain error".into()) }
                     else { Ok(x.ln()) } }
        "exp"   => Ok(x.exp()),
        "sqrt"  => { if x < 0.0 { Err("sqrt: negative".into()) }
                     else { Ok(x.sqrt()) } }
        "cbrt"  => Ok(x.cbrt()),
        "abs"   => Ok(x.abs()),
        "floor" => Ok(x.floor()),
        "ceil"  => Ok(x.ceil()),
        "round" => Ok(x.round()),
        "fact"  => {
            if x < 0.0 || (x - x.floor()).abs() > 1e-9 || x > 170.0 {
                return Err("fact: domain error".into());
            }
            Ok(factorial(x as u64) as f64)
        }
        _ => Err(format!("Unknown function '{name}'")),
    }
}

fn factorial(n: u64) -> u64 {
    (1..=n).fold(1u64, |acc, i| acc.saturating_mul(i))
}

// ── Public API ────────────────────────────────────────────────────────────

/// Evaluate a mathematical expression. `degrees` controls trig units.
pub fn evaluate(expr: &str, degrees: bool) -> Result<f64, String> {
    let toks = tokenise(expr)?;
    let mut p = Parser { toks: &toks, pos: 0, deg: degrees };
    let val = parse_expr(&mut p)?;
    if *p.peek() != Tok::End {
        return Err(format!("Unexpected input at position {}", p.pos));
    }
    Ok(val)
}

/// Format an f64 nicely for display (strips trailing zeros, uses sci notation
/// for very large or very small values).
pub fn fmt_num(n: f64) -> String {
    if n.is_nan()      { return "Error".into(); }
    if n.is_infinite() { return if n > 0.0 { "∞".into() } else { "-∞".into() }; }
    if n == 0.0        { return "0".into(); }

    let abs = n.abs();
    if abs >= 1e15 || (abs < 1e-9 && abs > 0.0) {
        return format!("{:.6e}", n);
    }
    if n.fract().abs() < 1e-10 && abs < 1e15 {
        return format!("{:.0}", n);
    }
    let s = format!("{:.10}", n);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        assert_eq!(evaluate("2+3",    false).unwrap(), 5.0);
        assert_eq!(evaluate("10-4",   false).unwrap(), 6.0);
        assert_eq!(evaluate("3*4",    false).unwrap(), 12.0);
        assert_eq!(evaluate("8/2",    false).unwrap(), 4.0);
        assert_eq!(evaluate("10%3",   false).unwrap(), 1.0);
    }

    #[test]
    fn precedence_and_parens() {
        let v = evaluate("2+3*4", false).unwrap();
        assert!((v - 14.0).abs() < 1e-10, "Expected 14, got {v}");
        let v = evaluate("(2+3)*4", false).unwrap();
        assert!((v - 20.0).abs() < 1e-10);
    }

    #[test]
    fn power_right_assoc() {
        let v = evaluate("2^3^2", false).unwrap(); // 2^(3^2) = 2^9 = 512
        assert!((v - 512.0).abs() < 1e-10, "Expected 512, got {v}");
    }

    #[test]
    fn trig_degrees() {
        let s = evaluate("sin(90)", true).unwrap();
        assert!((s - 1.0).abs() < 1e-10, "sin(90°) = {s}");
        let c = evaluate("cos(0)", true).unwrap();
        assert!((c - 1.0).abs() < 1e-10, "cos(0°) = {c}");
    }

    #[test]
    fn constants() {
        let p = evaluate("pi", false).unwrap();
        assert!((p - PI).abs() < 1e-10);
        let e = evaluate("e", false).unwrap();
        assert!((e - E).abs() < 1e-10);
    }

    #[test]
    fn factorial_val() {
        let f = evaluate("fact(5)", false).unwrap();
        assert!((f - 120.0).abs() < 1e-10);
    }

    #[test]
    fn fmt_num_cases() {
        assert_eq!(fmt_num(0.0),     "0");
        assert_eq!(fmt_num(1.0),     "1");
        assert_eq!(fmt_num(3.14),    "3.14");
        assert_eq!(fmt_num(1.5),     "1.5");
    }
}
