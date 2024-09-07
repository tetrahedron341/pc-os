use core2::io::{Read, Result as IoResult};

#[repr(C)]
#[repr(align(1))]
pub struct RawUstarHeader {
    pub file_name: [u8; 100],
    pub _file_mode: [u8; 8],
    pub _owner_uid: [u8; 8],
    pub _group_uid: [u8; 8],
    pub file_size: [u8; 12],
    pub _last_mod: [u8; 12],
    pub header_checksum: [u8; 8],
    pub file_type: u8,
    pub _linked_file_name: [u8; 100],
    pub ustar_indicator: [u8; 8],
    pub _owner_name: [u8; 32],
    pub _group_name: [u8; 32],
    pub _device_maj: [u8; 8],
    pub _device_min: [u8; 8],
    pub _file_name_prefix: [u8; 155],
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UstarFormatError {
    InvalidMagic,
    InvalidChecksum(u32, u32),
    InvalidType,
    UnexpectedNonOctalChar,
    SliceTooSmall,
}

pub struct UstarFile<'a> {
    raw_header: &'a RawUstarHeader,
    data: core2::io::Cursor<&'a [u8]>,
}

impl<'a> UstarFile<'a> {
    /// Returns a reader for a USTAR file placed in memory
    ///
    /// # Safety
    /// `ptr` must refer to a valid USTAR file header
    ///
    /// # TODO
    /// This could very easily buffer overflow.
    pub unsafe fn from_raw(ptr: *const RawUstarHeader) -> Self {
        let raw_header = &*ptr;
        let data = {
            let base = (ptr as *const u8).offset(512);
            let size = oct_to_u32(&raw_header.file_size).unwrap() as usize;
            core2::io::Cursor::new(core::slice::from_raw_parts(base, size))
        };
        UstarFile { raw_header, data }
    }

    pub fn read_raw(slice: &'a [u8]) -> Result<Self, UstarFormatError> {
        if slice.len() < 512 {
            return Err(UstarFormatError::SliceTooSmall);
        }
        // Safety: RawUstarHeader has no invalid states, and the index will panic anyways if it is out of bounds.
        let raw: &RawUstarHeader = unsafe { core::mem::transmute(&slice[0..512][0]) };
        if &raw.ustar_indicator != b"ustar  \0" {
            return Err(UstarFormatError::InvalidMagic);
        }

        if raw.file_type != 0 && !(b'0'..=b'6').contains(&raw.file_type) {
            return Err(UstarFormatError::InvalidType);
        }

        let checksum = oct_to_u32(&raw.header_checksum)?;
        let sum = slice
            .iter()
            .enumerate()
            .take(512)
            .map(|(i, byte)| {
                if (148..156).contains(&i) {
                    b' ' as u32 // Pretend checksum bytes are empty spaces
                } else {
                    *byte as u32
                }
            })
            .sum();
        if checksum != sum {
            return Err(UstarFormatError::InvalidChecksum(checksum, sum));
        }

        // Check to make sure all of the data is contained within the slice.
        let file_size = oct_to_u32(&raw.file_size)? as usize;
        if 512 + file_size >= slice.len() {
            return Err(UstarFormatError::SliceTooSmall);
        }

        Ok(UstarFile {
            raw_header: raw,
            data: core2::io::Cursor::new(&slice[512..512 + file_size]),
        })
    }

    // pub unsafe fn read_raw_ptr(ptr: *const u8) -> Result<Self, UstarFormatError> {
    //     let raw = &*(ptr as *const RawUstarHeader);
    //     if &raw.ustar_indicator != b"ustar  \0" {
    //         return Err(UstarFormatError::InvalidMagic);
    //     }

    //     if raw.file_type != 0 && !(b'0'..=b'6').contains(&raw.file_type) {
    //         return Err(UstarFormatError::InvalidType);
    //     }

    //     let checksum = oct_to_u32(&raw.header_checksum)?;
    //     let mut sum = 0;
    //     for i in 0..512 {
    //         if (148..156).contains(&i) {
    //             continue;
    //         }
    //         sum += *(ptr.offset(i)) as u32;
    //     }
    //     if checksum != sum {
    //         return Err(UstarFormatError::InvalidChecksum(checksum, sum));
    //     }

    //     Ok(Self::from_raw(ptr as *const RawUstarHeader))
    // }

    pub fn file_name(&self) -> alloc::borrow::Cow<str> {
        let file_name = {
            let mut s: &[u8] = &self.raw_header.file_name;
            for (i, &b) in s.iter().enumerate() {
                if b == 0 {
                    s = &s[..i];
                    break;
                }
            }
            s
        };
        alloc::string::String::from_utf8_lossy(file_name)
    }

    pub fn is_directory(&self) -> bool {
        self.raw_header.file_type == b'5'
    }

    pub fn is_file(&self) -> bool {
        self.raw_header.file_type == b'0' || self.raw_header.file_type == 0
    }

    pub fn file_size(&self) -> usize {
        oct_to_u32(&self.raw_header.file_size).unwrap() as usize
    }

    pub fn data(&self) -> &[u8] {
        self.data.get_ref()
    }
}

impl<'a> Read for UstarFile<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.data.read(buf)
    }
}

/// Expects a slice containing several USTAR entries
pub fn get_all_entries(mut data: &[u8]) -> alloc::vec::Vec<UstarFile> {
    let mut v = alloc::vec![];

    while let Ok(ustar) = UstarFile::read_raw(data) {
        let offset = 512 + ustar.file_size();
        let padding = if offset % 512 > 0 {
            512 - offset % 512
        } else {
            0
        };
        data = &data[offset + padding..];
        v.push(ustar);
    }

    v
}

fn oct_to_u32(oct: &[u8]) -> Result<u32, UstarFormatError> {
    let mut n = 0u32;
    for &d in oct {
        if d == 0 {
            break;
        } else if (b'0'..=b'7').contains(&d) {
            n <<= 3; // Multiply by 8
            n |= (d & 0x07) as u32; // Least significant three bits hold the value of the digit, so add them to our sum.
        } else {
            return Err(UstarFormatError::UnexpectedNonOctalChar);
        }
    }

    Ok(n)
}
