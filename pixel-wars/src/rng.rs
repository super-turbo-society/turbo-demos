use crate::*;
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct RNG {
    pub seed: u32,
}

impl RNG {
    pub fn new(seed: u32) -> Self {
        RNG { seed }
    }

    pub fn next(&mut self) -> u32 {
        let a: u32 = 1664525;
        let c: u32 = 1013904223;
        let m: u32 = u32::MAX;

        self.seed = (a.wrapping_mul(self.seed).wrapping_add(c)) % m;
        self.seed
    }

    pub fn next_in_range(&mut self, min: u32, max: u32) -> u32 {
        let range = max - min + 1;
        let mut number = (self.next() % range) + min;

        if range % 2 == 0 {
            number += 1;
        }

        number % range + min
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
}
