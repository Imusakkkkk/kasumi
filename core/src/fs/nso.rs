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