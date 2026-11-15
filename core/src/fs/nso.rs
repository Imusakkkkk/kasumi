#[derive(Debug, PartialEq, Eq)]
pub struct NsoHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub flags: u32,
    pub text_file_offset: u32,
    pub text_mem_offset: u32,
    pub text_size: u32,
    pub text_file_size: u32,
    pub ro_file_offset: u32,
    pub ro_mem_offset: u32,
    pub ro_size: u32,
    pub ro_file_size: u32,
    pub data_file_offset: u32,
    pub data_mem_offset: u32,
    pub data_size: u32,
    pub data_file_size: u32,
    pub bss_size: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NsoError {
    Truncated,
    BadMagic,
}

impl NsoHeader {
    fn extract_segment(&self, file: &[u8], off: u32, comp_size: u32, unc_size: u32) -> Result<Vec<u8>, NsoError> {
        let start = off as usize;
        let cs = comp_size as usize;
        let us = unc_size as usize;
        if start.checked_add(cs).map_or(true, |e| e > file.len()) { return Err(NsoError::Truncated); }
        let slice = &file[start..start + cs];
        if cs < us {
            let v = lz4_flex::decompress_size_prepended(slice).map_err(|_| NsoError::Truncated)?;
            if v.len() != us { return Err(NsoError::Truncated); }
            Ok(v)
        } else {
            Ok(slice.to_vec())
        }
    }

    pub fn extract_text(&self, file: &[u8]) -> Result<Vec<u8>, NsoError> {
        self.extract_segment(file, self.text_file_offset, self.text_file_size, self.text_size)
    }
    pub fn extract_ro(&self, file: &[u8]) -> Result<Vec<u8>, NsoError> {
        self.extract_segment(file, self.ro_file_offset, self.ro_file_size, self.ro_size)
    }
    pub fn extract_data(&self, file: &[u8]) -> Result<Vec<u8>, NsoError> {
        self.extract_segment(file, self.data_file_offset, self.data_file_size, self.data_size)
    }
}

#[derive(Debug)]
pub struct LoadedNso {
    pub entry_point: u64,
    pub text: Vec<u8>,
    pub ro: Vec<u8>,
    pub data: Vec<u8>,
    pub bss_size: u32,
    pub text_addr: u64,
    pub ro_addr: u64,
    pub data_addr: u64,
}

impl NsoHeader {
    pub fn load(&self, file: &[u8]) -> Result<LoadedNso, NsoError> {
        let text = self.extract_text(file)?;
        let ro = self.extract_ro(file)?;
        let data = self.extract_data(file)?;
        Ok(LoadedNso {
            entry_point: self.text_mem_offset as u64,
            text,
            ro,
            data,
            bss_size: self.bss_size,
            text_addr: self.text_mem_offset as u64,
            ro_addr: self.ro_mem_offset as u64,
            data_addr: self.data_mem_offset as u64,
        })
    }
}

pub fn parse_nso(data: &[u8]) -> Result<NsoHeader, NsoError> {
    if data.len() < 0x44 { return Err(NsoError::Truncated); }
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&data[0..4]);
    if magic != *b"NSO0" { return Err(NsoError::BadMagic); }
    Ok(NsoHeader {
        magic,
        version: u32::from_le_bytes(data[4..8].try_into().unwrap()),
        flags: u32::from_le_bytes(data[12..16].try_into().unwrap()),
        text_file_offset: u32::from_le_bytes(data[16..20].try_into().unwrap()),
        text_mem_offset: u32::from_le_bytes(data[20..24].try_into().unwrap()),
        text_size: u32::from_le_bytes(data[24..28].try_into().unwrap()),
        text_file_size: u32::from_le_bytes(data[28..32].try_into().unwrap()),
        ro_file_offset: u32::from_le_bytes(data[32..36].try_into().unwrap()),
        ro_mem_offset: u32::from_le_bytes(data[36..40].try_into().unwrap()),
        ro_size: u32::from_le_bytes(data[40..44].try_into().unwrap()),
        ro_file_size: u32::from_le_bytes(data[44..48].try_into().unwrap()),
        data_file_offset: u32::from_le_bytes(data[48..52].try_into().unwrap()),
        data_mem_offset: u32::from_le_bytes(data[52..56].try_into().unwrap()),
        data_size: u32::from_le_bytes(data[56..60].try_into().unwrap()),
        data_file_size: u32::from_le_bytes(data[60..64].try_into().unwrap()),
        bss_size: u32::from_le_bytes(data[64..68].try_into().unwrap()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn make_fixture() -> Vec<u8> {
        // 5 instructions: movz x0,#42; movz x1,#100; add x2,x0,x1; movz x3,#1; add x4,x2,x3
        let text: [u8; 20] = [
            0x40, 0x05, 0x80, 0xd2, // movz x0, #42
            0x81, 0x0c, 0x80, 0xd2, // movz x1, #100
            0x02, 0x00, 0x01, 0x8b, // add x2, x0, x1
            0x23, 0x00, 0x80, 0xd2, // movz x3, #1
            0x44, 0x00, 0x02, 0x8b, // add x4, x2, x3
        ];
        let ro = [0x11, 0x22];
        let data = [0x33, 0x44, 0x55, 0x66];
        let header_size = 0x44usize;
        let text_off = header_size;
        let ro_off = text_off + text.len();
        let data_off = ro_off + ro.len();
        let mut buf = vec![0u8; data_off + data.len()];
        buf[0..4].copy_from_slice(b"NSO0");
        // flags 0
        // text descriptor
        buf[16..20].copy_from_slice(&(text_off as u32).to_le_bytes());
        buf[20..24].copy_from_slice(&0x1000u32.to_le_bytes()); // mem offset
        buf[24..28].copy_from_slice(&(text.len() as u32).to_le_bytes()); // size
        buf[28..32].copy_from_slice(&(text.len() as u32).to_le_bytes()); // file size
        // ro
        buf[32..36].copy_from_slice(&(ro_off as u32).to_le_bytes());
        buf[36..40].copy_from_slice(&0x2000u32.to_le_bytes());
        buf[40..44].copy_from_slice(&(ro.len() as u32).to_le_bytes());
        buf[44..48].copy_from_slice(&(ro.len() as u32).to_le_bytes());
        // data
        buf[48..52].copy_from_slice(&(data_off as u32).to_le_bytes());
        buf[52..56].copy_from_slice(&0x3000u32.to_le_bytes());
        buf[56..60].copy_from_slice(&(data.len() as u32).to_le_bytes());
        buf[60..64].copy_from_slice(&(data.len() as u32).to_le_bytes());
        buf[64..68].copy_from_slice(&0u32.to_le_bytes()); // bss
        // copy segments
        buf[text_off..text_off + text.len()].copy_from_slice(&text);
        buf[ro_off..ro_off + ro.len()].copy_from_slice(&ro);
        buf[data_off..data_off + data.len()].copy_from_slice(&data);
        buf
    }

    #[test]
    fn nso_fixture_parses() {
        let data = make_fixture();
        let h = parse_nso(&data).unwrap();
        assert_eq!(h.magic, *b"NSO0");
        let t = h.extract_text(&data).unwrap();
        assert_eq!(t.len(), 20);
    }

    #[test]
    fn nso_fixture_loads_into_memory() {
        use crate::cpu::cpu_manager::load_nso_into;
        use crate::memory::translation::AddressSpace;
        let data = make_fixture();
        let h = parse_nso(&data).unwrap();
        let nso = h.load(&data).unwrap();
        let mut mem = vec![0u8; 0x4000];
        let mut aspace = AddressSpace::new();
        load_nso_into(&mut mem, &mut aspace, &nso).unwrap();
        assert_eq!(mem[0x1000], 0x40);
        assert_eq!(&mem[0x2000..0x2002], &[0x11, 0x22]);
        assert_eq!(&mem[0x3000..0x3004], &[0x33, 0x44, 0x55, 0x66]);
    }

    #[test]
    fn nso_bad_magic() {
        let mut data = vec![0u8; 0x44];
        data[0..4].copy_from_slice(b"XXXX");
        assert_eq!(parse_nso(&data), Err(NsoError::BadMagic));
    }

    #[test]
    fn nso_truncated() {
        let data = vec![0u8; 0x10];
        assert_eq!(parse_nso(&data), Err(NsoError::Truncated));
    }
}