use std::{fs, io::Error, path::PathBuf};

pub fn parse_row_fast(line: &str) -> Result<[u16; 96], Error> {
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

// this is the helper function that give the results in binary format
// for validation. Note that this is one way of doing the encoding
// However, the performance might be impact as storing a gigantic
// string before converting might be a problem
fn generate_test_output_validation(numbers: Vec<u16>) -> Vec<u8> {
    let binary_strings: Vec<String> = numbers
        .iter()
        .map(|&num| {
            let mut num_binary_str = format!("{:b}", num);
            if (2..=127).contains(&num) {
                let padding_zeros = 7 - num_binary_str.len();
                let padding_zeros_str = "0".repeat(padding_zeros);
                num_binary_str = format!("1110{}{}", padding_zeros_str, num_binary_str);
            } else if num > 127 {
                let padding_zeros = 11 - num_binary_str.len();
                let padding_zeros_str = "0".repeat(padding_zeros);
                num_binary_str = format!("1111{}{}", padding_zeros_str, num_binary_str);
            } else if num == 0 {
                num_binary_str = String::from("0");
            } else if num == 1 {
                num_binary_str = String::from("110");
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
