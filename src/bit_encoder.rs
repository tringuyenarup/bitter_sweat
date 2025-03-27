#[derive(Default)]
pub struct BitEncoder {
    bytes: Vec<u8>,
    current_byte: u8,
    bits_filled: u8,
}

impl BitEncoder {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            current_byte: 0u8,
            bits_filled: 0u8,
        }
    }

    pub fn encode_bits(&mut self, encoded_value: u16, num_bits: u8) {
        let mask = (1u16 << num_bits) - 1;
        let mut encode_value_in_bits = encoded_value & mask;
        let current_byte_remaining_bits = 8 - self.bits_filled;

        if num_bits <= current_byte_remaining_bits {
            self.current_byte |=
                (encode_value_in_bits << (current_byte_remaining_bits - num_bits)) as u8;
            self.bits_filled += num_bits;

            if self.bits_filled == 8 {
                self.bytes.push(self.current_byte);
                self.current_byte = 0;
                self.bits_filled = 0;
            }
        } else {
            let num_bits_to_write = num_bits;

            if self.bits_filled > 0 {
                let shift_to_remove = num_bits_to_write - current_byte_remaining_bits;

                self.current_byte |= ((encode_value_in_bits >> shift_to_remove)
                    & ((1 << current_byte_remaining_bits) - 1))
                    as u8;

                self.bytes.push(self.current_byte);
                self.current_byte = 0;
                self.bits_filled = 0;
                encode_value_in_bits &= (1 << shift_to_remove) - 1;
                println!("encode_value_in_bits &= (1 << shift_to_remove) - 1;");
                println!("{:b}", encode_value_in_bits);
            }

            let mut bits_left = num_bits_to_write - current_byte_remaining_bits;
            println!("bits_left= {:?}", bits_left);
            while bits_left >= 8 {
                let shift = bits_left - 8;
                let next_byte = encode_value_in_bits >> shift;
                println!("{:b}", next_byte);
                self.bytes.push(next_byte as u8);
                encode_value_in_bits &= (1 << shift) - 1;
                bits_left -= 8;
            }

            if bits_left > 0 {
                self.current_byte = (encode_value_in_bits << (8 - bits_left)) as u8;
                self.bits_filled = bits_left;
            }
        }
    }
    pub fn flush(&mut self) {
        if self.bits_filled > 0 {
            self.bytes.push(self.current_byte);
            self.current_byte = 0;
            self.bits_filled = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_value_for_7_bits() {
        let encoder = encode_test_data(&[42]);
        // 1110 0101010 (11 bits) = 42 (1110 + 0101010)
        assert_eq!(encoder.bytes, vec![0b11100101, 0b01000000]);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_single_value_for_11_bits() {
        let encoder = encode_test_data(&[1000]);

        assert_eq!(encoder.bytes, vec![0b11110111, 0b11010000]);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_mixed_values() {
        // Test a mix of different value types
        let encoder = encode_test_data(&[42, 1000]);
        // Bit pattern:
        // 1110 0101010 (11 bits) = 42 (1110 + 0101010)
        // 1111 01111101000(15 bits) = 1000 (1111 + 00011111010)
        // Total: 32 bits = 4 bytes
        assert_eq!(
            encoder.bytes,
            vec![0b11100101, 0b01011110, 0b11111010, 0b00000000]
        );
        assert_eq!(encoder.bits_filled, 0);
    }
    #[test]
    fn test_small_value_encoding() {
        let encoder = encode_test_data(&[2]);
        assert_eq!(encoder.bytes, vec![0b11100000, 0b01000000]);
        assert_eq!(encoder.current_byte, 0);
        assert_eq!(encoder.bits_filled, 0);
    }
    fn encode_test_data(values: &[u16]) -> BitEncoder {
        let mut encoder = BitEncoder::new();
        for &v in values {
            match v {
                // 0
                0 => encoder.encode_bits(0, 1),
                // 10
                u16::MAX => encoder.encode_bits(2, 2),
                // 110
                1 => encoder.encode_bits(6, 3),
                other_value => {
                    if (2..=127).contains(&other_value) {
                        encoder.encode_bits(14, 4);
                        encoder.encode_bits(other_value, 7);
                    } else {
                        encoder.encode_bits(15, 4);
                        encoder.encode_bits(other_value, 11);
                    }
                }
            }
        }
        encoder.flush();
        encoder
    }

    #[test]
    fn test_zero_encoding() {
        let encoder = encode_test_data(&[0, 0, 0, 0]);
        assert_eq!(encoder.bytes, vec![0b00000000]);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_one_encoding() {
        let encoder = encode_test_data(&[1, 1]);
        assert_eq!(encoder.bytes, vec![0b11011000]);
        assert_eq!(encoder.current_byte, 0);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_max_value_encoding() {
        let encoder = encode_test_data(&[u16::MAX]);
        assert_eq!(encoder.bytes, vec![0b10000000]);
        assert_eq!(encoder.bits_filled, 0);
    }
    #[test]
    fn test_large_value_encoding() {
        let encoder = encode_test_data(&[128]); // 1111 00010000000 (15 bits)
        assert_eq!(
            encoder.bytes,
            vec![
                0b11110001, // 1111 0001 (first 8 bits)
                0b00000000  // 0000000 0 (last 7 bits + 1 padding zero)
            ]
        );
        assert_eq!(encoder.bits_filled, 0);
    }
}
