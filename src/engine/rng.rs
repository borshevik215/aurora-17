#[derive(Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xA17A2026
            } else {
                seed
            },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;

        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;

        self.state = x;

        x.wrapping_mul(
            0x2545_F491_4F6C_DD1D
        )
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next_u64() as f32
            / u64::MAX as f32
    }

    pub fn range_f32(
        &mut self,
        min: f32,
        max: f32,
    ) -> f32 {
        min + self.next_f32()
            * (max - min)
    }

    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        min + (self.next_u64() as u32 % (max - min + 1))
    }

    pub fn range_usize(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_u64() as usize % (max - min + 1))
    }
}
