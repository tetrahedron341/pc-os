pub static SCREEN: spin::Once<spin::Mutex<Screen>> = spin::Once::new();

/// # Safety
/// `framebuffer` must satisfy the same constraints as [`Screen::new`]
pub unsafe fn init_screen(framebuffer: &'static mut bootloader::boot_info::FrameBuffer) {
    SCREEN.call_once(move || spin::Mutex::new(Screen::new(framebuffer)));
}

pub fn lock_screen<'a>() -> Option<spin::MutexGuard<'a, Screen>> {
    SCREEN.get().map(|s| s.lock())
}

pub struct Screen {
    framebuffer: &'static mut bootloader::boot_info::FrameBuffer,
}

impl Screen {
    /// Initialize a new screen using a framebuffer recieved from the bootloader
    ///
    /// # Safety
    /// `framebuffer` must refer to a valid framebuffer obtained from the bootloader, and there must be no other living references to the framebuffer
    pub unsafe fn new(framebuffer: &'static mut bootloader::boot_info::FrameBuffer) -> Self {
        Screen { framebuffer }
    }

    pub fn height(&self) -> usize {
        self.framebuffer.info().vertical_resolution
    }

    pub fn width(&self) -> usize {
        self.framebuffer.info().horizontal_resolution
    }

    #[inline]
    fn calculate_pixel_index(&self, x: usize, y: usize) -> usize {
        let info = self.framebuffer.info();
        (y * info.stride + x) * info.bytes_per_pixel
    }

    /// Draw a single pixel to the screen. Out-of-bounds pixels will be silently ignored.
    #[inline]
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: impl Color) {
        if y >= self.height() || x >= self.width() {
            return;
        }
        let color = u32::to_be_bytes(color.as_argb_u32());
        let offset = self.calculate_pixel_index(x, y);
        if let [r, g, b, _a] = &mut self.framebuffer.buffer_mut()[offset..offset + 4] {
            *r = color[1];
            *g = color[2];
            *b = color[3];
        }
    }

    /// Draw a rectangle. Color values will be provided by a function, for ease of use with fonts, textures, etc.
    /// Out-of-bounds pixels will be silently ignored and the color function will not be called for them.
    pub fn draw_rect_with<F, C>(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        mut colors: F,
    ) where
        F: FnMut(usize, usize, &Self) -> C,
        C: Color,
    {
        for py in y..y + height {
            for px in x..x + width {
                self.draw_pixel(px, py, colors(px, py, self));
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if y >= self.height() || x >= self.width() {
            return None;
        }
        let offset = self.calculate_pixel_index(x, y);
        if let [r, g, b, a] = self.framebuffer.buffer()[offset..offset + 4] {
            Some(u32::from_be_bytes([a, r, g, b]))
        } else {
            None
        }
    }
}

pub trait Color {
    fn as_argb_u32(&self) -> u32;
}

impl Color for (u8, u8, u8) {
    fn as_argb_u32(&self) -> u32 {
        let (r, g, b) = (self.0 as u32, self.1 as u32, self.2 as u32);
        r << 16 | g << 8 | b
    }
}

impl Color for u32 {
    fn as_argb_u32(&self) -> u32 {
        self & 0x00FFFFFF
    }
}

pub mod console {
    use super::super::font;
    use super::*;

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
        bg_color: u32,
    }

    impl<'a> Console<'a> {
        pub fn new(screen_mutex: &'a spin::Mutex<Screen>) -> Self {
            let (width, height) = {
                let screen = screen_mutex.lock();
                (screen.width(), screen.height())
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
                _ => return,
            };
            let gx = self.cursor.col * font::FONT.width();
            let gy = self.cursor.row * font::FONT.height();
            self.screen.lock().draw_rect_with(
                gx,
                gy,
                font::FONT.width(),
                font::FONT.height(),
                |x, y, _| {
                    let x = x - gx;
                    let y = y - gy;
                    if bitmap[y * bytes_per_row + x / 8] & (1 << (7 - (x % 8))) != 0 {
                        self.cursor.fg_color
                    } else {
                        self.cursor.bg_color
                    }
                },
            );
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
            let mut screen = self.screen.lock();
            let width = screen.width();
            let height = screen.height();
            screen.draw_rect_with(0, 0, width, height, |x, y, screen| {
                screen
                    .get_pixel(x, y + font::FONT.height())
                    .unwrap_or(self.cursor.bg_color)
            });
        }
    }

    impl<'a> core::fmt::Write for Console<'a> {
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
}
