#![allow(dead_code)]
//! Bounded LIFO calculation history (newest first, max 100 items).
use chrono::Local;

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    pub expr:   String,
    pub result: String,
    pub ts:     String,
}

impl HistoryRecord {
    fn new(expr: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            expr:   expr.into(),
            result: result.into(),
            ts:     Local::now().format("%H:%M:%S").to_string(),
        }
    }
}

pub struct History {
    records: Vec<HistoryRecord>,
    max:     usize,
}

impl History {
    pub fn new() -> Self { Self { records: Vec::new(), max: 100 } }

    pub fn push(&mut self, expr: impl Into<String>, result: impl Into<String>) {
        self.records.insert(0, HistoryRecord::new(expr, result));
        self.records.truncate(self.max);
    }

    pub fn all(&self)  -> &[HistoryRecord] { &self.records }
    pub fn clear(&mut self)                { self.records.clear(); }
    pub fn len(&self)  -> usize            { self.records.len() }
    pub fn is_empty(&self) -> bool         { self.records.is_empty() }

    pub fn get(&self, idx: usize) -> Option<&HistoryRecord> { self.records.get(idx) }
}

impl Default for History { fn default() -> Self { Self::new() } }
