use std::{io::{BufReader, Read, Seek}, fs::File};
use encore_shared::FileFormat;

static FILE_HEADERS: &[(&[u8], u64)] = &[
    ( b"OggS", 0 ),          // .ogg
    ( b"ID3", 0 ),           // .mp3
    ( b"\xff\xfb\x90", 0 ),  // .mp3 (MPEG ADTS)
    ( b"fLaC", 0 ),          // .flac
    ( b"RIFF", 0 ),          // .wav
];

/// because `file-format` is bloated
/// i did it in 25 SLOC
pub fn check_file(file: &mut BufReader<File>) -> Result<FileFormat, Box<dyn std::error::Error>> {
    use std::io::SeekFrom;

    let mut ret: FileFormat = FileFormat::Other;
    for (header, header_offset) in FILE_HEADERS {
        let mut buf = vec![0; header.len()];
        if file.seek(SeekFrom::Start(*header_offset)).is_err() {
            // possibly out of bounds
            continue;
        }
        file.read_exact(&mut buf)?;
        if buf != *header {
            continue;
        }

        ret = FileFormat::Audio;
        break;
    }
    file.seek(SeekFrom::Start(0))?;

    Ok(ret)
}

