use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use std::fs::{self, File};
use std::io::Error;
use std::path::{Path, PathBuf};

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

fn parse_row_fast(line: &str) -> Result<[u16; 96], Error> {
    let mut counts = [0u16; 96];
    let byte_iter = line.as_bytes().iter().enumerate();
    let mut count_idx = 0;
    let mut num = 0u16;

    let mut commas_seen = 0;

    for (_, &byte) in byte_iter {
        if byte == b',' {
            commas_seen += 1;
            if commas_seen <= 2 {
                continue; // Skip detector and date
            }
            if count_idx < 96 {
                counts[count_idx] = num;
                count_idx += 1;
                num = 0;
            }
        } else if commas_seen >= 2 && byte.is_ascii_digit() {
            num = num * 10 + (byte - b'0') as u16;
        }
    }

    if count_idx < 96 {
        counts[count_idx] = num;
    }

    if count_idx != 96 {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected exactly 96 counts",
        ));
    }
    Ok(counts)
}

fn encode_file(input_path: &Path, binary_encoder: &mut BinaryEncoder) -> Result<(), Error> {
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
                // 0 values
                0u16 => binary_encoder.encode_bits(bits, num_bits),
                // error values
                u16::MAX => binary_encoder.encode_bits(bits, num_bits),
                1u16..=120u16 => binary_encoder.encode_bits(bits, num_bits),
                _ => binary_encoder.encode_bits(bits, num_bits),
            }
        }
    }

    Ok(())
}

fn parse_paths(all_refined_counts_path: &str) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = fs::read_dir(all_refined_counts_path)?
        .map(|res| res.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|path| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse::<u16>().ok())
                .is_some()
        })
        .collect();

    paths.sort_by_key(|path| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0)
    });
    Ok(paths)
}

fn main() -> Result<(), Error> {
    let encode_path = "output.bin";
    let mut writer = File::create(encode_path)?;
    let mut binary_encoder = BinaryEncoder::new();

    // let all_refined_counts_path = "../output";
    // let all_files = parse_paths(all_refined_counts_path)?;

    // for file in all_files.iter() {
    //     encode(file, &mut writer)?;
    //     println!("Finish {:?}", file);
    // }
    // for (path, summary) in all_paths.iter().zip(all_summaries.iter()) {
    //     encode(path, &mut writer, summary)?;
    //     println!("Finish {:?}", path);
    // }

    encode_file(Path::new("../output/100.csv"), &mut binary_encoder)?;

    Ok(())
}

struct BinaryEncoder {
    bytes: Vec<u8>,
    current_byte: u8,
    bits_filled: u8,
}

impl BinaryEncoder {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            current_byte: 0u8,
            bits_filled: 0u8,
        }
    }

    fn encode_bits(&mut self, bits: u8, num_bits: u8) {
        let mask = 1 << (num_bits - 1);
        let bits = bits & mask;

        let remaining_bits = 8 - self.bits_filled;

        if num_bits <= remaining_bits {
            self.current_byte != bits << (remaining_bits - num_bits);
            self.bits_filled += num_bits;

            if self.bits_filled == 8 {
                self.bytes.push(self.current_byte.clone());
                self.current_byte = 0u8;
                self.bits_filled = 0u8;
            }
        }
    }
}
