use alloc::collections::BTreeMap;

pub struct Font {
    psf: &'static [u8],
}

pub static FONT: Font = Font::from_psf(include_bytes!("../../fonts/ter-v16n.psf"));

lazy_static::lazy_static! {
    pub static ref GLYPH_MAP: BTreeMap<char, usize> = FONT.generate_glyph_map();
}

impl Font {
    const fn from_psf(file: &'static [u8]) -> Self {
        Font { psf: file }
    }

    pub fn width(&self) -> usize {
        match self.psf_version() {
            PsfVersion::Psf2 => {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&self.psf[28..32]);
                u32::from_le_bytes(bytes) as usize
            }
            PsfVersion::Psf1 => 8,
            _ => panic!("Invalid PSF file"),
        }
    }

    pub fn height(&self) -> usize {
        match self.psf_version() {
            PsfVersion::Psf2 => {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&self.psf[24..28]);
                u32::from_le_bytes(bytes) as usize
            }
            PsfVersion::Psf1 => self.psf[3] as usize,
            _ => panic!("Invalid PSF file"),
        }
    }

    pub fn charsize(&self) -> usize {
        match self.psf_version() {
            PsfVersion::Psf2 => {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&self.psf[20..24]);
                u32::from_le_bytes(bytes) as usize
            }
            PsfVersion::Psf1 => self.psf[3] as usize,
            _ => panic!("Invalid PSF file"),
        }
    }

    fn header_len(&self) -> usize {
        match self.psf_version() {
            PsfVersion::Psf2 => {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&self.psf[8..12]);
                u32::from_le_bytes(bytes) as usize
            }
            PsfVersion::Psf1 => 4,
            _ => panic!("Invalid PSF file"),
        }
    }

    fn glyph_count(&self) -> usize {
        match self.psf_version() {
            PsfVersion::Psf2 => {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&self.psf[16..20]);
                u32::from_le_bytes(bytes) as usize
            }
            PsfVersion::Psf1 => {
                if self.psf[2] & 0x01 != 0 {
                    512
                } else {
                    256
                }
            }
            _ => panic!("Invalid PSF file"),
        }
    }

    const fn has_unicode_table(&self) -> bool {
        if let PsfVersion::Psf1 = self.psf_version() {
            self.psf[2] & 0x02 != 0
        } else {
            self.psf[12] & 0x01 != 0
        }
    }

    const fn psf_version(&self) -> PsfVersion {
        if self.psf[0] == 0x72 && self.psf[1] == 0xb5 && self.psf[2] == 0x4a && self.psf[3] == 0x86
        {
            PsfVersion::Psf2
        } else if self.psf[0] == 0x36 && self.psf[1] == 0x04 {
            PsfVersion::Psf1
        } else {
            PsfVersion::Invalid
        }
    }

    fn generate_glyph_map(&self) -> BTreeMap<char, usize> {
        if !self.has_unicode_table() {
            return BTreeMap::new();
        }
        match self.psf_version() {
            PsfVersion::Psf2 => self.generate_glyph_map_psf2(),
            PsfVersion::Psf1 => self.generate_glyph_map_psf1(),
            _ => panic!("Invalid PSF file"),
        }
    }

    fn generate_glyph_map_psf2(&self) -> BTreeMap<char, usize> {
        enum State {
            ZeroBytes,

            OneByteOutofTwo(u8),

            OneByteOutofThree(u8),
            TwoBytesOutofThree(u8, u8),

            OneByteOutofFour(u8),
            TwoBytesOutofFour(u8, u8),
            ThreeBytesOutofFour(u8, u8, u8),

            Seq,
        }
        let table_offset = self.header_len() + self.charsize() * self.glyph_count();
        let mut gid = 0;
        let mut offset = table_offset;
        let mut state = State::ZeroBytes;
        let mut map = BTreeMap::new();
        let mut map_char = |c: u32, g: usize| unsafe {
            map.insert(char::from_u32_unchecked(c), g);
        };
        while offset < self.psf.len() && gid < self.glyph_count() {
            let byte = self.psf[offset];
            offset += 1;
            match state {
                State::ZeroBytes => match byte {
                    0xFF => {
                        gid += 1;
                    }
                    0xFE => {
                        state = State::Seq;
                    }
                    0b0000_0000..=0b0111_1111 => map_char(byte as u32, gid),
                    0b1100_0000..=0b1101_1111 => state = State::OneByteOutofTwo(byte),
                    0b1110_0000..=0b1110_1111 => state = State::OneByteOutofThree(byte),
                    0b1111_0000..=0b1111_0111 => state = State::OneByteOutofFour(byte),
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::OneByteOutofTwo(b1) => match byte {
                    0b1000_0000..=0b1011_1111 => {
                        state = State::ZeroBytes;
                        map_char(((b1 & 0x1F) as u32) << 6 | ((byte & 0x3F) as u32), gid)
                    }
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::OneByteOutofThree(b1) => match byte {
                    0b1000_0000..=0b1011_1111 => state = State::TwoBytesOutofThree(b1, byte),
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::TwoBytesOutofThree(b1, b2) => match byte {
                    0b1000_0000..=0b1011_1111 => {
                        state = State::ZeroBytes;
                        map_char(
                            ((b1 & 0x0F) as u32) << 12
                                | ((b2 & 0x3F) as u32) << 6
                                | ((byte & 0x3F) as u32),
                            gid,
                        )
                    }
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::OneByteOutofFour(b1) => match byte {
                    0b1000_0000..=0b1011_1111 => state = State::TwoBytesOutofFour(b1, byte),
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::TwoBytesOutofFour(b1, b2) => match byte {
                    0b1000_0000..=0b1011_1111 => state = State::ThreeBytesOutofFour(b1, b2, byte),
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::ThreeBytesOutofFour(b1, b2, b3) => match byte {
                    0b1000_0000..=0b1011_1111 => {
                        state = State::ZeroBytes;
                        map_char(
                            ((b1 & 0x07) as u32) << 18
                                | ((b2 & 0x3F) as u32) << 12
                                | ((b3 & 0x3F) as u32) << 6
                                | ((byte & 0x3F) as u32),
                            gid,
                        )
                    }
                    _ => panic!("Malformed PSF File (Invalid UTF8 at {:#X})", offset),
                },
                State::Seq => {
                    if byte == 0xFF {
                        state = State::ZeroBytes;
                        gid += 1;
                    }
                }
            }
        }
        map
    }

    fn generate_glyph_map_psf1(&self) -> BTreeMap<char, usize> {
        enum State {
            Direct,
            Seq,
        }
        let table_offset = self.header_len() + self.charsize() * self.glyph_count();
        let mut gid = 0;
        let mut offset = table_offset;
        let mut state = State::Direct;
        let mut map = BTreeMap::new();
        while offset < self.psf.len() && gid < self.glyph_count() {
            match state {
                State::Direct => {
                    let mut code = [0u8; 2];
                    code.copy_from_slice(&self.psf[offset..offset + 2]);
                    let code = u16::from_le_bytes(code);
                    offset += 2;
                    if code == 0xFFFF {
                        gid += 1;
                    } else if code == 0xFFFE {
                        state = State::Seq;
                    } else {
                        let c = char::decode_utf16(Some(code)).next().unwrap().unwrap();
                        map.insert(c, gid);
                    }
                }
                State::Seq => {
                    let mut code = [0u8; 2];
                    code.copy_from_slice(&self.psf[offset..offset + 2]);
                    let code = u16::from_le_bytes(code);
                    offset += 2;
                    if code == 0xFFFF {
                        gid += 1;
                        state = State::Direct
                    } // We dont support character sequences
                }
            }
        }
        map
    }

    pub fn str_to_glyphs<'a>(&self, s: &'a str) -> impl Iterator<Item = usize> + 'a {
        let has_unicode_table = self.has_unicode_table();
        s.chars().map(move |c| {
            if has_unicode_table {
                GLYPH_MAP.get(&c).copied().unwrap_or(0x91)
            } else if c.is_ascii() {
                c as usize
            } else {
                0x91
            }
        })
    }

    /// On a successful query, this returns a tuple with the (bytes per row, bitmap) of the glyph.
    pub fn get_glyph_bitmap(&self, gid: usize) -> Result<(usize, &[u8]), &'static str> {
        if gid >= self.glyph_count() {
            return Err("Glyph ID out of range");
        }
        let offset = self.header_len() + gid * self.charsize();
        Ok((
            self.charsize() / self.height(),
            &self.psf[offset..offset + self.charsize()],
        ))
    }
}

#[derive(Eq, PartialEq)]
enum PsfVersion {
    Psf1,
    Psf2,
    Invalid,
}
