use binary_writer::bit_encoder::BitEncoder;
#[allow(unused_imports)]
use std::fs::{self, File};
use std::io::Error;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
#[allow(unused_variables)]
#[allow(dead_code)]
fn parse_line(line: &str) -> [u16; 96] {
    let counts_vec = line
        .split(',')
        .skip(2)
        .take(96)
        .map(|c| match c.parse::<u16>() {
            Ok(n) => n,
            Err(_) => u16::MAX,
        })
        .collect::<Vec<u16>>();
    counts_vec.try_into().expect("Expected exactly 96 counts")
}
#[allow(dead_code)]
fn encode_file(input_path: &Path, bit_encoder: &mut BitEncoder) -> Result<(), Error> {
    let file = fs::read_to_string(input_path)?;

    let mut skip_header = true;

    for line in file.lines() {
        if skip_header {
            skip_header = false;
            continue;
        }
        let row: [u16; 96] = parse_line(line);

        for v in row.into_iter() {
            match v {
                // 0 values - encode as 0 - 1 bit
                0u16 => bit_encoder.encode_bits(0, 1),
                // error values - encode as 10 - 2 bits
                u16::MAX => bit_encoder.encode_bits(2, 2),
                // 1 values - encode as 110 - 3 bits
                1 => bit_encoder.encode_bits(6, 3),
                // other values
                other_value => {
                    // values from 2 -> 127 will be prefix with
                    // 1110xxxxxxx
                    if (2..=127).contains(&other_value) {
                        bit_encoder.encode_bits(14, 4);
                        bit_encoder.encode_bits(other_value, 7);
                    } else {
                        // values from 128 and above will be prefix with
                        // 1111xxxxxxx
                        bit_encoder.encode_bits(15, 4);
                        bit_encoder.encode_bits(other_value, 11);
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    // let encode_path = "output.bin";
    // let mut writer = File::create(encode_path)?;
    // let mut bit_encoder = BitEncoder::new();

    // encode_file(Path::new("../output/100.csv"), &mut binary_encoder)?;

    Ok(())
}

// use super::*;
