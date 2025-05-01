pub mod file_format;

use std::fs::File;
use std::io::BufReader;

/// skip_num parameter for skipping the first n elements in the vec.
/// needed cuz ./encore 1.mp3 will make argv = ["encore", "1.mp3"], and try to parse argv[0] = "encore"
// pub fn parse_playlist(file: &Vec<String>, skip_num: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
pub fn parse_playlist(file: &[String], skip_num: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    for s in file.iter()
        .skip(skip_num)
        .by_ref()
    {
        let f = if let Ok(k) = File::open(s) { k } else {
            eprintln!("{s} doesn't exist");
            continue
        };
        let mut f = BufReader::new(f);
        if file_format::check_file(&mut f)? != encore_shared::FileFormat::Audio {
            eprintln!("Removing {s} from playlist: not audio file. skipping...");
            continue;
        }
        lines.push(s.to_owned());
    }

    Ok(lines)
}
