use core::sync::atomic::{AtomicU32, Ordering};

pub static SCREEN: spin::Once<spin::Mutex<Screen>> = spin::Once::new();

pub unsafe fn init_screen(graphics_mode: &'static bootloader::bootinfo::VesaGraphicsMode) {
    SCREEN.call_once(|| {
        spin::Mutex::new(Screen::new(graphics_mode))
    });
}

pub fn lock_screen<'a>() -> Option<spin::MutexGuard<'a, Screen>> {
    SCREEN.r#try().map(|s| s.lock())
}

pub struct Screen {
    framebuffer: &'static mut [AtomicU32],
    width: usize,
    height: usize,
}

impl Screen {
    pub unsafe fn new(graphics_mode: &'static bootloader::bootinfo::VesaGraphicsMode) -> Self {
        let framebuffer = { 
            let base = graphics_mode.framebuffer as usize as *mut AtomicU32;
            let size = graphics_mode.pitch as usize * graphics_mode.height as usize;
            core::slice::from_raw_parts_mut(base, size)
        };
        let width = graphics_mode.width as usize;
        let height = graphics_mode.height as usize;

        Screen {
            framebuffer, width, height
        }
    }

    /// Draw a single pixel to the screen. Out-of-bounds pixels will be silently ignored.
    #[inline]
    pub fn draw_pixel(&self, x: usize, y: usize, color: impl Color) {
        if y >= self.height || x >= self.width {
            return
        }
        let color = color.as_argb_u32();
        let offset = y * self.width + x;
        self.framebuffer[offset].store(color, Ordering::Relaxed);
    }

    /// Draw a rectangle. Color values will be provided by a function, for ease of use with fonts, textures, etc.
    /// Out-of-bounds pixels will be silently ignored and the color function will not be called for them.
    pub fn draw_rect_with<F,C>(&self, x: usize, y: usize, width: usize, height: usize, mut colors: F) 
        where F: FnMut(usize,usize) -> C,
              C: Color
    {
        for row in y..y+height {
            let row_offset = row * self.width;
            for col in x..x+width {
                self.framebuffer[row_offset + col].store(colors(col,row).as_argb_u32(), Ordering::Relaxed);
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if y >= self.height || x >= self.width {
            return None
        }
        let offset = y * self.width + x;
        Some(self.framebuffer[offset].load(Ordering::Relaxed))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

pub trait Color {
    fn as_argb_u32(self) -> u32;
}

impl Color for (u8,u8,u8) {
    fn as_argb_u32(self) -> u32 {
        let (r,g,b) = (self.0 as u32, self.1 as u32, self.2 as u32);
        r << 16 |
        g << 8 |
        b
    }
}

impl Color for u32 {
    fn as_argb_u32(self) -> u32 {
        self & 0x00FFFFFF
    }
}

pub mod console {
    use super::*;
    use super::super::font;

    pub struct Console<'a> {
        screen: &'a spin::Mutex<Screen>,
        rows: usize,
        columns: usize,
        cursor: Cursor,
    }
    
    struct Cursor {
        row: usize,
        col: usize,
        fg_color: u32,
        bg_color: u32
    }
    
    impl<'a> Console<'a> {
        pub fn new(screen_mutex: &'a spin::Mutex<Screen>) -> Self {
            let (width, height) = {
                let screen = screen_mutex.lock();
                (screen.width, screen.height)
            };
            Console {
                rows: height / font::FONT.height(),
                columns: width / font::FONT.width(),
                cursor: Cursor {
                    row: 0,
                    col: 0,
                    fg_color: 0xffffffff,
                    bg_color: 0x00000000,
                },
                screen: screen_mutex,
            }
        }

        pub fn write_glyph(&mut self, gid: usize) {
            let (bytes_per_row, bitmap) = match font::FONT.get_glyph_bitmap(gid) {
                Ok(g) => g,
                _ => return
            };
            let gx = self.cursor.col * font::FONT.width();
            let gy = self.cursor.row * font::FONT.height();
            self.screen.lock().draw_rect_with(
                gx, 
                gy, 
                font::FONT.width(), 
                font::FONT.height(), 
                |x,y| {
                    let x = x - gx;
                    let y = y - gy;
                    if bitmap[y*bytes_per_row+x/8] & (1<<(7-(x%8))) != 0{
                        self.cursor.fg_color
                    } else {
                        self.cursor.bg_color
                    }
                });
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
            self.cursor.row -= 1;
            let screen = self.screen.lock();
            screen.draw_rect_with(
                0, 
                0, 
                screen.width(), 
                screen.height(),
                |x,y| {
                    screen.get_pixel(x, y + font::FONT.height()).unwrap_or(self.cursor.bg_color)
                });
        }
    }
    
    impl<'a> core::fmt::Write for Console<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let mut lines = s.split('\n');
            // Print the first line without a newline()
            lines.next().map(|line| {
                for glyph in font::FONT.str_to_glyphs(line) {
                    self.write_glyph(glyph);
                }
            });
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
}