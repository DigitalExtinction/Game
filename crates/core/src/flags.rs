/// Bit flags.
#[derive(Default)]
pub struct Flags(u32);

impl Flags {
    /// Changes value of a specific bit.
    pub fn set(&mut self, bit: u32, value: bool) {
        let mask = 1 << bit;
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }

    /// Returns value of a specific bit flag.
    pub fn get(&self, bit: u32) -> bool {
        self.0 & (1 << bit) != 0
    }

    /// Returns true if any of the bites is non-zero.
    pub fn any(&self) -> bool {
        self.0 > 0
    }
}
