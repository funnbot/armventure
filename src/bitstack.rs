use bit::BitCt;

pub fn push_bits_u32(current: u32, value: u32, len: u8) -> u32 {
    let mask = !(u32::MAX << len);
    let masked = value & mask;
    (current << len) | masked
}

pub fn push_bits_offset_u32(current: u32, value: u32, bit_len: u8, offset: u8) -> u32 {
    let unshifted = current.rotate_left(offset as u32);
    let pushed = push_bits_u32(unshifted, value, bit_len);
    pushed.rotate_right((offset + bit_len) as u32)
}

// #[derive(Clone, Copy)]
// pub struct BitStack<Num, const MAX_LEN: u8> {
//     value: Num,
//     len: u8,
// }

// impl<const MAX_LEN: u8> BitStack<u32, MAX_LEN> {
//     pub fn new() -> Self {
//         Self { value: 0, len: 0 }
//     }
//     pub fn push(self, value: u32, bit_len: u8) -> Self {
//         assert!(self.len + bit_len <= MAX_LEN);
//         Self {
//             value: push_bits_u32(self.value, value, bit_len),
//             len: self.len + bit_len,
//         }
//     }
// }

pub struct BitStackU32 {
    value: u32,
    len: u8,
}

impl BitStackU32 {
    pub fn new() -> Self {
        Self { value: 0, len: 0 }
    }
    pub fn push(&mut self, value: u32, bit_len: u8) {
        assert!(self.len + bit_len <= 32);
        self.len += bit_len;
        self.value = push_bits_u32(self.value, value, bit_len);
    }
    pub fn all_bits_written(&self) -> bool {
        self.len == 32
    }
    /// for push_bits_offset_u32, the offset is len before the push
    pub fn len(&self) -> u8 {
        self.len
    }
    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn fill_zero(&mut self) {
        let rem = 32 - self.len;
        self.push(0, rem);
    }
}

#[cfg(test)]
mod tests {
    use std::process::Termination;

    use super::*;

    #[test]
    fn push_bits_u32_works() {
        assert_eq!(push_bits_u32(0b1111, 0b0110, 4), 0b11110110);
        assert_eq!(
            push_bits_offset_u32(0b1111000011110000, 0b01100, 5, 16 + 4),
            0b1111011001110000u32
        );
        assert_eq!(
            push_bits_offset_u32(0b1111000011110000, 0b01100, 5, 16 + 4),
            0b1111011001110000u32
        );
        assert_eq!(
            push_bits_offset_u32(0b1111000011110000, 0b0110, 4, 16 + 4),
            0b1111011011110000u32
        );
        let mut v = 0;
        v = push_bits_u32(v, 0b1111, 4);
        let idx = 4;
        v = push_bits_u32(v, 0b101, 3);
        v = push_bits_u32(v, 0, 32 - 7);
        assert_eq!(v, 0b11111010_00000000_00000000_00000000);
        v = push_bits_offset_u32(v, 0b010, 3, idx);
        assert_eq!(v, 0b11110100_00000000_00000000_00000000);
    }
}
