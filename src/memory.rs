//! Single memory register: MC / MR / M+ / M- / MS
#[derive(Debug, Default, Clone)]
pub struct Memory { value: f64 }

impl Memory {
    pub fn new() -> Self { Self { value: 0.0 } }
    pub fn clear(&mut self)          { self.value = 0.0; }
    pub fn recall(&self)   -> f64    { self.value }
    pub fn store(&mut self, v: f64)  { self.value = v; }
    pub fn add(&mut self,   v: f64)  { self.value += v; }
    pub fn sub(&mut self,   v: f64)  { self.value -= v; }
    pub fn has_value(&self)-> bool   { self.value != 0.0 }
}
