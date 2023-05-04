use core::fmt;
use spin::Mutex;

pub const VGA_TEXT_BUFFER: *mut u8 = 0xb8000 as *mut u8;

lazy_static::lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        style: Style::from_colors(FgColor::White, BgColor::Black, false),
        column_position: 0,
        buffer: unsafe {&mut *(VGA_TEXT_BUFFER as *mut VgaBuffer)},
    });
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum FgColor {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum BgColor {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct Style(u8);

#[allow(dead_code)]
impl Style {
    pub fn from_byte(byte: u8) -> Self {
        Style(byte)
    }

    pub fn as_byte(self) -> u8 {
        self.0
    }

    pub fn from_colors(fg_color: FgColor, bg_color: BgColor, blink: bool) -> Self {
        Style(
            if blink {0b1000_0000} else {0} |
            ((bg_color as u8) << 4) |
            (fg_color as u8)
        )
    }

    pub fn with_fg_color(self, fg_color: FgColor) -> Self {
        Style(
            (self.0 & 0b1111_0000) | fg_color as u8
        )
    }

    pub fn with_bg_color(self, bg_color: BgColor) -> Self {
        Style(
            (self.0 & 0b1000_1111) | ((bg_color as u8) << 4)
        )
    }

    pub fn with_blink(self, blink: bool) -> Self {
        Style(
            (self.0 & 0b0111_1111) | (if blink {0b1000_000} else {0})
        )
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(C)]
pub struct ScreenChar {
    pub char: u8,
    pub style: Style,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

#[repr(transparent)]
pub struct VgaBuffer {
    pub chars: [[volatile::Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    pub column_position: usize,
    pub style: Style,
    pub buffer: &'static mut VgaBuffer
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let style = self.style;

                self.buffer.chars[row][col].write(ScreenChar {char: byte, style});
                self.column_position += 1;
                if self.column_position < BUFFER_WIDTH {self.move_cursor(row, col+1)};
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe)
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(char);
            }
        }
        self.clear_row(BUFFER_HEIGHT-1);
        self.column_position = 0;
        self.move_cursor(BUFFER_HEIGHT - 1, 0);
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {char: b' ', style: self.style};
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn enable_cursor(&mut self) {
        unsafe {
            let mut addrp = x86_64::instructions::port::Port::<u8>::new(0x3D4);
            let mut datap = x86_64::instructions::port::Port::<u8>::new(0x3D5);
            addrp.write(0x0A);                                  //outb(0x3D4, 0x0A);
            let cursor_start = (datap.read() & 0xC0) | 13;
            datap.write(cursor_start);                          //outb(0x3D5, (inb(0x3D5) & 0xC0) | cursor_start);
        
            addrp.write(0x0B);                                  //outb(0x3D4, 0x0B);
            let cursor_end = (datap.read() & 0xE0) | 15;
            datap.write(cursor_end);                            //outb(0x3D5, (inb(0x3D5) & 0xE0) | cursor_end);
        }
    }

    pub fn disable_cursor(&mut self) {
        unsafe {
            let mut addrp = x86_64::instructions::port::Port::<u8>::new(0x3D4);
            let mut datap = x86_64::instructions::port::Port::<u8>::new(0x3D5);

            addrp.write(0x0A);
            datap.write(0x20);
        }
    }

    pub fn move_cursor(&mut self, row: usize, col: usize) {
        assert!(row < BUFFER_HEIGHT && col < BUFFER_WIDTH);
        unsafe {
            let mut addrp = x86_64::instructions::port::Port::<u8>::new(0x3D4);
            let mut datap = x86_64::instructions::port::Port::<u8>::new(0x3D5);
            let pos = row * 80 + col;                    // uint16_t pos = y * VGA_WIDTH + x;
     
            addrp.write(0x0F);                                  // outb(0x3D4, 0x0F);
            datap.write((pos & 0xff) as u8);                    // outb(0x3D5, (uint8_t) (pos & 0xFF));
            addrp.write(0x0E);                                  // outb(0x3D4, 0x0E);
            datap.write(((pos >> 8) & 0xff) as u8);             // outb(0x3D5, (uint8_t) ((pos >> 8) & 0xFF));
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_single() {
    println!("test_println_single output")
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output")
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    x86_64::instructions::interrupts::without_interrupts(|| {
        use fmt::Write;
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).unwrap();
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.char), c);
        }
    });
}