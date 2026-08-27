//! Minimal Chromium data-pack (.pak v5) reader/writer (port of `pak.py`).
//!
//! Layout (little-endian):
//!   header:  u32 version(=5) | u32 encoding | u16 num_entries | u16 num_aliases
//!   index:   (num_entries + 1) x { u16 resource_id | u32 offset }
//!            (the last record is a sentinel with id 0 whose offset is EOF)
//!   aliases: num_aliases x { u16 resource_id | u16 index }
//!   payload: raw resource bytes (gzip-compressed for most text resources)

use std::path::Path;

pub struct Pak {
    pub data: Vec<u8>,
    /// (resource_id, offset), including the trailing sentinel (id 0).
    pub entries: Vec<(u16, u32)>,
    /// (resource_id, index) alias records.
    pub aliases: Vec<(u16, u16)>,
}

fn u16_at(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

fn u32_at(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

pub fn parse(path: &Path) -> Result<Pak, String> {
    let data = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    if data.len() < 12 {
        return Err("pak too small".into());
    }
    let version = u32_at(&data, 0);
    if version != 5 {
        return Err(format!("unsupported pak version: {version}"));
    }
    let num_entries = u16_at(&data, 8) as usize;
    let num_aliases = u16_at(&data, 10) as usize;
    let mut entries = Vec::with_capacity(num_entries + 1);
    let mut off = 12usize;
    for _ in 0..num_entries + 1 {
        entries.push((u16_at(&data, off), u32_at(&data, off + 2)));
        off += 6;
    }
    let mut aliases = Vec::with_capacity(num_aliases);
    for _ in 0..num_aliases {
        aliases.push((u16_at(&data, off), u16_at(&data, off + 2)));
        off += 4;
    }
    Ok(Pak { data, entries, aliases })
}

impl Pak {
    /// (resource_id, payload_bytes) for every entry, in file order.
    pub fn blobs(&self) -> Vec<(u16, &[u8])> {
        let mut out = Vec::with_capacity(self.entries.len().saturating_sub(1));
        for i in 0..self.entries.len().saturating_sub(1) {
            let (rid, start) = self.entries[i];
            let end = self.entries[i + 1].1;
            out.push((rid, &self.data[start as usize..end as usize]));
        }
        out
    }
}

/// Rebuild a pak from [(id, bytes), ...] (file order) and the alias table.
pub fn build(blobs: &[(u16, Vec<u8>)], aliases: &[(u16, u16)], encoding: u32) -> Vec<u8> {
    let num_entries = blobs.len() as u16;
    let num_aliases = aliases.len() as u16;
    let mut out = Vec::new();
    out.extend_from_slice(&5u32.to_le_bytes());
    out.extend_from_slice(&encoding.to_le_bytes());
    out.extend_from_slice(&num_entries.to_le_bytes());
    out.extend_from_slice(&num_aliases.to_le_bytes());
    let mut offset = 12u32 + (num_entries as u32 + 1) * 6 + num_aliases as u32 * 4;
    let mut body = Vec::new();
    for (rid, blob) in blobs {
        out.extend_from_slice(&rid.to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        body.extend_from_slice(blob);
        offset += blob.len() as u32;
    }
    // sentinel
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&offset.to_le_bytes());
    for (rid, idx) in aliases {
        out.extend_from_slice(&rid.to_le_bytes());
        out.extend_from_slice(&idx.to_le_bytes());
    }
    out.extend_from_slice(&body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let blobs: Vec<(u16, Vec<u8>)> = vec![
            (1, b"hello".to_vec()),
            (2, vec![0x1f, 0x8b, 1, 2, 3]),
            (300, vec![]),
        ];
        let aliases: Vec<(u16, u16)> = vec![(301, 0), (302, 2)];
        let bytes = build(&blobs, &aliases, 1);
        let dir = std::env::temp_dir().join(format!("pak-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.pak");
        std::fs::write(&path, &bytes).unwrap();
        let pak = parse(&path).unwrap();
        assert_eq!(pak.entries.len(), 4); // 3 + sentinel
        assert_eq!(pak.entries.last().unwrap().0, 0);
        assert_eq!(pak.entries.last().unwrap().1 as usize, bytes.len());
        assert_eq!(pak.aliases, aliases);
        let got: Vec<(u16, Vec<u8>)> =
            pak.blobs().into_iter().map(|(id, b)| (id, b.to_vec())).collect();
        assert_eq!(got, blobs);
        // byte-identical rebuild
        assert_eq!(build(&got, &pak.aliases, 1), bytes);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Roundtrip the REAL resources.pak of the installed app (read-only):
    /// parse -> rebuild must be byte-identical, and the Doubao page
    /// detection heuristic must find its pages.
    #[test]
    fn real_resources_pak_roundtrip() {
        let versions = std::path::Path::new(
            "/Applications/DoubaoWork.app/Contents/Helpers/DoubaoWork Browser.app/\
             Contents/Frameworks/DoubaoWork Browser Framework.framework/Versions",
        );
        if !versions.exists() {
            eprintln!("skipping: {versions:?} not found");
            return;
        }
        let mut dirs: Vec<_> = std::fs::read_dir(versions)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name().and_then(|n| n.to_str()) != Some("Current")
                    && p.symlink_metadata().map(|m| !m.file_type().is_symlink()).unwrap_or(false)
            })
            .collect();
        if dirs.len() != 1 {
            eprintln!("skipping: unexpected framework versions: {dirs:?}");
            return;
        }
        let pak_path = dirs.remove(0).join("Resources/resources.pak");
        let pak = parse(&pak_path).unwrap();
        let blobs: Vec<(u16, Vec<u8>)> =
            pak.blobs().into_iter().map(|(id, b)| (id, b.to_vec())).collect();
        let original = std::fs::read(pak_path).unwrap();
        assert_eq!(build(&blobs, &pak.aliases, 1), original, "pak rebuild not lossless");

        // heuristic check: some entries gunzip to Doubao html pages
        let mut pages = 0;
        for (_, blob) in &blobs {
            if blob.starts_with(b"\x1f\x8b") {
                use std::io::Read as _;
                let mut raw = Vec::new();
                if flate2::read::GzDecoder::new(&blob[..]).read_to_end(&mut raw).is_ok() {
                    let head = &raw[..raw.len().min(4000)];
                    let lowered: Vec<u8> = raw
                        .iter()
                        .skip_while(|b| b.is_ascii_whitespace())
                        .map(|b| b.to_ascii_lowercase())
                        .collect();
                    if lowered.starts_with(b"<!doctype")
                        && head.windows(6).any(|w| w == b"og:url")
                        && head.windows(6).any(|w| w == b"doubao")
                    {
                        pages += 1;
                    }
                }
            }
        }
        assert!(pages > 0, "no Doubao pages detected in real pak");
        eprintln!("real pak: {} entries, {} doubao pages", blobs.len(), pages);
    }
}
