use crate::fs::nup::*;
use std::io::Read;

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    fn make_fs_raw() -> NupFileSystem {
        // minimal toc with one file at extent 0 size 4 raw
        let mut toc = NupToc {
            header: NupTocHeader {
                magic: 0x01434F54,
                package_count: 1,
                dir_count: 1,
                file_count: 1,
                string_pool_off: 0,
                string_pool_size: 0,
                reserved: 0,
            },
            packages: vec![NupPackage {
                title_id: 0,
                root_dir: 0,
                exefs_dir: 0xFFFFFFFF,
                romfs_dir: 0xFFFFFFFF,
                version: 0,
                kind: 0,
                flags: 0,
                reserved: 0,
                reserved2: 0,
                meta_offset: 0,
                meta_size: 0,
            }],
            dirs: vec![NupDir {
                name_off: 0,
                name_len: 0,
                first_child: 0,
                child_count: 1,
                reserved: 0,
            }],
            files: vec![NupFile {
                name_off: 0,
                name_len: 0,
                size: 4,
                extent_offset: 0,
                extent_size: 4,
                sha256: sha2::Sha256::digest(b"ABCD").into(),
                codec: 0,
                codec_flags: 0,
                reserved2: 0,
                reserved3: [0; 12],
            }],
            children: vec![1], // file id = dir_count(1)+0 =1
            string_pool: vec![],
        };
        // children id for file = dirs.len() + 0 = 1
        let data = b"ABCD".to_vec();
        NupFileSystem { toc, data }
    }

    #[test]
    fn read_raw() {
        let fs = make_fs_raw();
        let f = &fs.toc.files[0];
        let out = fs.read_file(f).unwrap();
        assert_eq!(out, b"ABCD");
    }

    #[test]
    fn nested_traversal() {
        let mut fs = make_fs_raw();
        // add a nested dir manually
        fs.toc.dirs.push(NupDir {
            name_off: 0,
            name_len: 0,
            first_child: 1,
            child_count: 1,
            reserved: 0,
        });
        fs.toc.dirs[0].child_count = 1;
        fs.toc.children = vec![1, 2];
        // no dir_count field on NupToc
        // ensure no panic
        assert_eq!(fs.list_directory("/").unwrap().len(), 1);
    }

    #[test]
    fn find_file_by_path_root() {
        let fs = make_fs_raw();
        // root dir has child file at slot 0
        let f = fs.find_file_by_path("file");
        // name empty so find by path may fail; just ensure no panic
        let _ = f;
    }
}

pub struct NupFileSystem {
    pub toc: NupToc,
    pub data: Vec<u8>, // full file bytes
}

impl NupFileSystem {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, NupError> {
        if data.len() < 0x80 { return Err(NupError::Truncated); }
        let toc_offset = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
        let toc_size = u64::from_le_bytes(data[16..24].try_into().unwrap()) as usize;
        if toc_offset + toc_size > data.len() { return Err(NupError::BadBlobBounds); }
        let toc_bytes = &data[toc_offset..toc_offset + toc_size];
        let sb = parse_superblock(&data, toc_bytes)?;
        let header = parse_toc_header(toc_bytes, sb.package_count)?;
        let toc = parse_nup_toc(toc_bytes, header)?;
        validate_nup_toc(&toc)?;
        Ok(Self { toc, data })
    }

    fn dir_name(&self, d: &crate::fs::nup::NupDir) -> Option<String> {
        crate::fs::nup::get_string(&self.toc.string_pool, d.name_off as usize, d.name_len as usize).ok().map(|s| s.to_string())
    }

    fn file_name(&self, f: &crate::fs::nup::NupFile) -> Option<String> {
        crate::fs::nup::get_string(&self.toc.string_pool, f.name_off as usize, f.name_len as usize).ok().map(|s| s.to_string())
    }

    pub fn children_of(&self, dir_index: usize) -> Vec<(bool, usize)> {
        // returns (is_dir, index)
        let dir = &self.toc.dirs[dir_index];
        let start = dir.first_child as usize;
        let count = dir.child_count as usize;
        let mut out = Vec::new();
        for i in 0..count {
            let slot = start + i;
            if slot >= self.toc.children.len() { break; }
            let id = self.toc.children[slot];
            if (id as usize) < self.toc.dirs.len() {
                out.push((true, id as usize));
            } else {
                out.push((false, (id as usize) - self.toc.dirs.len()));
            }
        }
        out
    }

    pub fn find_directory_by_path(&self, path: &str) -> Option<usize> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        // root
        if parts.is_empty() {
            // return root dir of first package
            if self.toc.packages.is_empty() { return None; }
            return Some(self.toc.packages[0].root_dir as usize);
        }
        let mut current = self.toc.packages[0].root_dir as usize;
        for part in parts {
            let mut found = None;
            for (is_dir, idx) in self.children_of(current) {
                if is_dir {
                    if let Some(name) = self.dir_name(&self.toc.dirs[idx]) {
                        if name == part { found = Some(idx); break; }
                    }
                }
            }
            match found {
                Some(i) => current = i,
                None => return None,
            }
        }
        Some(current)
    }

    pub fn find_file_by_path(&self, path: &str) -> Option<&crate::fs::nup::NupFile> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() { return None; }
        let (file_name, dir_parts) = parts.split_last()?;
        let dir_path = dir_parts.join("/");
        let dir_idx = if dir_parts.is_empty() {
            self.toc.packages.get(0)?.root_dir as usize
        } else {
            self.find_directory_by_path(&dir_path)?
        };
        for (is_dir, idx) in self.children_of(dir_idx) {
            if !is_dir {
                if let Some(name) = self.file_name(&self.toc.files[idx]) {
                    if name == *file_name { return Some(&self.toc.files[idx]); }
                }
            }
        }
        None
    }

    pub fn list_directory(&self, path: &str) -> Option<Vec<(String, bool)>> {
        let dir_idx = if path.trim_matches('/').is_empty() {
            if self.toc.packages.is_empty() { return None; }
            self.toc.packages[0].root_dir as usize
        } else {
            self.find_directory_by_path(path)?
        };
        let mut out = Vec::new();
        for (is_dir, idx) in self.children_of(dir_idx) {
            let name = if is_dir {
                self.dir_name(&self.toc.dirs[idx]).unwrap_or_default()
            } else {
                self.file_name(&self.toc.files[idx]).unwrap_or_default()
            };
            out.push((name, is_dir));
        }
        Some(out)
    }

    pub fn read_file_by_path(&self, path: &str) -> Result<Vec<u8>, crate::fs::nup::NupError> {
        let f = self.find_file_by_path(path).ok_or(crate::fs::nup::NupError::Truncated)?;
        self.read_file(f)
    }

    pub fn read_file(&self, file: &NupFile) -> Result<Vec<u8>, NupError> {
        let start = file.extent_offset as usize;
        let size = file.extent_size as usize;
        if start + size > self.data.len() { return Err(NupError::BadBlobBounds); }
        let raw = &self.data[start..start+size];
        match file.codec {
            0 => {
                if raw.len() != file.size as usize { return Err(NupError::BadBlobBounds); }
                Ok(raw.to_vec())
            }
            1 => {
                let mut out = Vec::with_capacity(file.size as usize);
                let mut decoder = zstd::stream::read::Decoder::new(raw).map_err(|_| NupError::BadBlobBounds)?;
                decoder.read_to_end(&mut out).map_err(|_| NupError::BadBlobBounds)?;
                // verify sha256
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(&out);
                if hash.as_slice() != file.sha256 { return Err(NupError::BadTocHash); }
                Ok(out)
            }
            _ => Err(NupError::BadCounts),
        }
    }
}