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

    pub fn encode_bits(&mut self, value: u16, num_bits: u8) {
        // Ensure we only use the specified number of bits from the value
        let mask = (1u16 << num_bits) - 1;
        let mut remaining_value = value & mask;

        // Calculate how many bits we can fit in the current byte
        let available_bits = 8 - self.bits_filled;

        if num_bits <= available_bits {
            // Simple case: all bits fit in the current byte
            self.current_byte |= (remaining_value << (available_bits - num_bits)) as u8;
            self.bits_filled += num_bits;

            // If we've filled the byte, push it and reset
            if self.bits_filled == 8 {
                self.bytes.push(self.current_byte);
                self.current_byte = 0;
                self.bits_filled = 0;
            }
        } else {
            // Complex case: bits span multiple bytes

            // 1. Fill the current byte if it's partially filled or even if it is empty
            // as the byte might start fresh here as well
            if available_bits > 0 {
                let bits_for_current = remaining_value >> (num_bits - available_bits);
                self.current_byte |= bits_for_current as u8;
                self.bytes.push(self.current_byte);

                // Update the remaining value by removing the bits we just used
                remaining_value &= (1 << (num_bits - available_bits)) - 1;
            }

            // 2. Write complete bytes while we have 8+ bits remaining
            let mut bits_remaining = num_bits - available_bits;
            while bits_remaining >= 8 {
                let next_byte = (remaining_value >> (bits_remaining - 8)) as u8;
                self.bytes.push(next_byte);
                bits_remaining -= 8;
            }

            // 3. Store any leftover bits in the current byte
            if bits_remaining > 0 {
                self.current_byte = (remaining_value << (8 - bits_remaining)) as u8;
                self.bits_filled = bits_remaining;
            } else {
                self.current_byte = 0;
                self.bits_filled = 0;
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

    pub fn encode(&mut self, values: &[u16]) {
        for &v in values {
            match v {
                // 0
                0 => self.encode_bits(0, 1),
                // 10
                u16::MAX => self.encode_bits(2, 2),
                // 110
                1 => self.encode_bits(6, 3),
                other_value => {
                    // 1110xxxxxxx
                    if (2..=127).contains(&other_value) {
                        self.encode_bits(14, 4);
                        self.encode_bits(other_value, 7);
                    } else {
                        // 111xxxxxxxxxxxx
                        self.encode_bits(15, 4);
                        self.encode_bits(other_value, 11);
                    }
                }
            }
        }
    }
}

// this is the helper function that give the results in binary format
// for validation. Note that this is one way of doing the encoding
// However, the performance might be impact as storing a gigantic
// string before converting might be a problem
fn generate_test_output_validation(numbers: &[u16]) -> Vec<u8> {
    let binary_strings: Vec<String> = numbers
        .iter()
        .map(|&num| {
            let mut num_binary_str = format!("{:b}", num);
            if num == 0 {
                num_binary_str = String::from("0");
            } else if num == 1 {
                num_binary_str = String::from("110");
            } else if (2..=127).contains(&num) {
                let padding_zeros = 7 - num_binary_str.len();
                let padding_zeros_str = "0".repeat(padding_zeros);
                num_binary_str = format!("1110{}{}", padding_zeros_str, num_binary_str);
            } else if 127 < num && num < u16::MAX {
                let padding_zeros = 11 - num_binary_str.len();
                let padding_zeros_str = "0".repeat(padding_zeros);
                num_binary_str = format!("1111{}{}", padding_zeros_str, num_binary_str);
            } else {
                num_binary_str = String::from("10");
            }
            num_binary_str
        })
        .collect();

    let combined_binary = binary_strings.join("");

    (0..combined_binary.len())
        .step_by(8)
        .map(|i| {
            let end = std::cmp::min(i + 8, combined_binary.len());
            let mut chunk = combined_binary[i..end].to_string();

            if chunk.len() < 8 {
                chunk.push_str(&"0".repeat(8 - chunk.len()));
            }

            u8::from_str_radix(&chunk, 2).unwrap()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_multiple_all_negative_values() {
        let mut encoder = BitEncoder::new();
        let input = vec![
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535,
            65535, 65535, 65535, 65535,
        ];
        let results = generate_test_output_validation(&input);
        encoder.encode(&input);
        encoder.flush();

        assert_eq!(encoder.bytes, results);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_multiple_random_large_values() {
        let mut encoder = BitEncoder::new();
        let input = vec![
            51, 39, 38, 33, 35, 30, 25, 18, 20, 16, 16, 11, 15, 9, 11, 12, 7, 9, 16, 11, 10, 14,
            22, 30, 21, 30, 32, 44, 35, 36, 53, 63, 56, 61, 71, 74, 79, 82, 97, 114, 101, 99, 98,
            104, 102, 109, 107, 120, 113, 123, 115, 118, 102, 89, 118, 113, 105, 91, 98, 92, 101,
            87, 110, 99, 83, 95, 91, 84, 81, 107, 90, 86, 68, 80, 75, 76, 65, 68, 58, 62, 49, 59,
            61, 52, 75, 68, 57, 62, 56, 58, 60, 49, 48, 40, 40, 32,
        ];
        let results = generate_test_output_validation(&input);
        encoder.encode(&input);
        encoder.flush();

        assert_eq!(encoder.bytes, results);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_multiple_random_with_negative() {
        let mut encoder = BitEncoder::new();
        let input = vec![
            8, 10, 7, 8, 3, 3, 6, 4, 3, 1, 5, 0, 5, 2, 2, 3, 2, 2, 5, 4, 1, 13, 17, 25, 36, 47, 66,
            49, 67, 62, 82, 86, 109, 115, 102, 103, 85, 109, 105, 120, 103, 65535, 65535, 65535,
            65535, 65535, 102, 113, 116, 127, 124, 119, 114, 122, 118, 115, 134, 125, 114, 132,
            152, 171, 160, 159, 167, 158, 153, 161, 156, 177, 141, 134, 131, 112, 98, 79, 82, 72,
            71, 66, 59, 49, 42, 45, 35, 41, 45, 38, 38, 31, 29, 16, 28, 15, 19, 4,
        ];
        let results = generate_test_output_validation(&input);
        encoder.encode(&input);
        encoder.flush();

        assert_eq!(encoder.bytes, results);
        assert_eq!(encoder.bits_filled, 0);
    }
    #[test]
    fn test_multiple_random_values() {
        let mut encoder = BitEncoder::new();
        let input = vec![
            14, 27, 28, 48, 37, 32, 35, 26, 17, 14, 11, 8, 11, 13, 13, 6, 8, 6, 5, 9, 4, 5, 4, 9,
            4, 12, 16, 18, 13, 14, 26, 22, 15, 18, 35, 31, 24, 30, 42, 43, 46, 72, 64, 57, 62, 59,
            68, 53, 67, 69, 71, 75, 55, 73, 58, 60, 57, 56, 79, 60, 71, 71, 61, 71, 57, 63, 61, 61,
            71, 57, 69, 62, 75, 58, 62, 63, 51, 57, 52, 54, 43, 49, 42, 43, 45, 61, 40, 34, 51, 37,
            33, 31, 23, 19, 16, 12,
        ];
        let results = generate_test_output_validation(&input);
        encoder.encode(&input);
        encoder.flush();

        assert_eq!(encoder.bytes, results);
        assert_eq!(encoder.bits_filled, 0);
    }

    #[test]
    fn test_multiple_zeros() {
        let mut encoder = BitEncoder::new();
        encoder.encode(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        encoder.flush();
        assert_eq!(
            encoder.bytes,
            vec![
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000
            ]
        );
        assert_eq!(encoder.bits_filled, 0);
    }
    #[test]
    fn test_single_value_for_7_bits() {
        let mut encoder = BitEncoder::new();
        encoder.encode(&[42]);
        encoder.flush();

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
        let encoder = encode_test_data(&[42, 1000]);
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
