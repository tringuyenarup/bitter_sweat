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
