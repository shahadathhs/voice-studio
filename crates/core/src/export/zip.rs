//! A minimal zip writer: entries are stored without compression, which is all a `.docx` needs
//! (Word reads it fine) and avoids a dependency. Only what the exporters write is supported:
//! ASCII names and files under 4 GB.

/// CRC-32 (the zip and PNG polynomial) of `data`.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn size(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn short(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

/// 1980-01-01 00:00, the earliest time a zip can hold: entries carry no meaningful date.
const DOS_TIME: u16 = 0;
const DOS_DATE: u16 = 0x0021;
/// Version needed to extract: 2.0.
const VERSION: u16 = 20;

/// The bytes of a zip archive holding `files` (name, contents), in order.
pub fn store(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, data) in files {
        let offset = size(out.len());
        let crc = crc32(data);
        out.extend(0x0403_4b50_u32.to_le_bytes());
        out.extend(VERSION.to_le_bytes());
        out.extend([0, 0, 0, 0]); // flags, method: stored
        out.extend(DOS_TIME.to_le_bytes());
        out.extend(DOS_DATE.to_le_bytes());
        out.extend(crc.to_le_bytes());
        out.extend(size(data.len()).to_le_bytes());
        out.extend(size(data.len()).to_le_bytes());
        out.extend(short(name.len()).to_le_bytes());
        out.extend([0, 0]); // extra field length
        out.extend(name.as_bytes());
        out.extend(*data);

        directory.extend(0x0201_4b50_u32.to_le_bytes());
        directory.extend(VERSION.to_le_bytes()); // made by
        directory.extend(VERSION.to_le_bytes()); // needed
        directory.extend([0, 0, 0, 0]); // flags, method
        directory.extend(DOS_TIME.to_le_bytes());
        directory.extend(DOS_DATE.to_le_bytes());
        directory.extend(crc.to_le_bytes());
        directory.extend(size(data.len()).to_le_bytes());
        directory.extend(size(data.len()).to_le_bytes());
        directory.extend(short(name.len()).to_le_bytes());
        // Extra field and comment lengths, disk number, internal and external attributes.
        directory.extend([0; 12]);
        directory.extend(offset.to_le_bytes());
        directory.extend(name.as_bytes());
    }
    let start = size(out.len());
    out.extend(&directory);
    out.extend(0x0605_4b50_u32.to_le_bytes());
    out.extend([0, 0, 0, 0]); // this disk, disk with the directory
    out.extend(short(files.len()).to_le_bytes());
    out.extend(short(files.len()).to_le_bytes());
    out.extend(size(directory.len()).to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend([0, 0]); // comment length
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_the_standard_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn the_archive_has_headers_a_directory_and_an_end_record() {
        let zip = store(&[("a.txt", b"hello"), ("dir/b.txt", b"world!")]);
        assert_eq!(&zip[..4], b"PK\x03\x04");
        // Local header 30 bytes + name + data.
        assert_eq!(&zip[30..35], b"a.txt");
        assert_eq!(&zip[35..40], b"hello");
        let end = zip.len() - 22;
        assert_eq!(&zip[end..end + 4], b"PK\x05\x06");
        assert_eq!(u16::from_le_bytes([zip[end + 10], zip[end + 11]]), 2);
        let directory = u32::from_le_bytes(zip[end + 16..end + 20].try_into().unwrap()) as usize;
        // Each directory entry is 46 bytes plus its name, and says where its file starts.
        let first = &zip[directory..];
        assert_eq!(&first[..4], b"PK\x01\x02");
        assert_eq!(&first[46..51], b"a.txt");
        assert_eq!(u32::from_le_bytes(first[42..46].try_into().unwrap()), 0);
        let second = &first[51..];
        assert_eq!(&second[..4], b"PK\x01\x02");
        assert_eq!(&second[46..55], b"dir/b.txt");
        let offset = u32::from_le_bytes(second[42..46].try_into().unwrap()) as usize;
        assert_eq!(&zip[offset..offset + 4], b"PK\x03\x04");
        // The directory ends exactly where the end record starts.
        assert_eq!(directory + 51 + 55, end);
    }
}
