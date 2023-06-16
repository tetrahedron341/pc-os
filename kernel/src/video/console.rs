use super::font;
use super::framebuffer::{GfxRectangle, Pixel};
use super::Framebuffer;
use alloc::boxed::Box;

pub static CONSOLE: spin::Mutex<Option<Console<Box<dyn Framebuffer + Send>>>> =
    spin::Mutex::new(None);

pub struct Console<F: Framebuffer> {
    fb: F,
    rows: usize,
    columns: usize,
    cursor: Cursor,
}

struct Cursor {
    row: usize,
    col: usize,
    fg_color: u32,
    bg_color: u32,
}

impl<F: Framebuffer> Console<F> {
    pub fn new(fb: F) -> Self {
        let (width, height) = (fb.info().width as usize, fb.info().height as usize);
        Console {
            rows: height / font::FONT.height(),
            columns: width / font::FONT.width(),
            cursor: Cursor {
                row: 0,
                col: 0,
                fg_color: 0xffffffff,
                bg_color: 0x000000ff,
            },
            fb,
        }
    }

    pub fn get_framebuffer(&self) -> &F {
        &self.fb
    }

    pub fn get_framebuffer_mut(&mut self) -> &mut F {
        &mut self.fb
    }

    pub fn write_glyph(&mut self, gid: usize) {
        let (bytes_per_row, bitmap) = match font::FONT.get_glyph_bitmap(gid) {
            Ok(g) => g,
            _ => return,
        };
        let gx = self.cursor.col * font::FONT.width();
        let gy = self.cursor.row * font::FONT.height();
        let rect = GfxRectangle::with(
            font::FONT.width() as u32,
            font::FONT.height() as u32,
            |x, y| {
                let x = x as usize;
                let y = y as usize;
                if bitmap[y * bytes_per_row + x / 8] & (1 << (7 - (x % 8))) != 0 {
                    Pixel::from_u32_rgba(self.cursor.fg_color)
                } else {
                    Pixel::from_u32_rgba(self.cursor.bg_color)
                }
            },
        );
        self.fb.blit(&rect, (gx as i32, gy as i32));
        self.cursor.col += 1;
        if self.cursor.col >= self.columns {
            self.newline()
        }
    }

    pub fn newline(&mut self) {
        self.cursor.col = 0;
        self.cursor.row += 1;
        if self.cursor.row >= self.rows {
            self.scroll_down()
        }
    }

    pub fn scroll_down(&mut self) {
        // self.cursor.row -= 1;
        // let mut screen = self.screen.lock();
        // let width = screen.width();
        // let height = screen.height();
        // screen.draw_rect_with(0, 0, width, height, |x, y, screen| {
        //     screen
        //         .get_pixel(x, y + font::FONT.height())
        //         .unwrap_or(self.cursor.bg_color)
        // });
    }
}

impl<F: Framebuffer> core::fmt::Write for Console<F> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut lines = s.split('\n');
        // Print the first line without a newline()
        if let Some(line) = lines.next() {
            for glyph in font::FONT.str_to_glyphs(line) {
                self.write_glyph(glyph);
            }
        }
        // Every line afterwards is preceded by a newline()
        for line in lines {
            self.newline();
            for glyph in font::FONT.str_to_glyphs(line) {
                self.write_glyph(glyph);
            }
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(console) = CONSOLE.lock().as_mut() {
            use core::fmt::Write;
            console.write_fmt(args).unwrap();
        }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::video::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
