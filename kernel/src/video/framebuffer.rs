use core::fmt::Display;
use core::ops::{Index, IndexMut};

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

/// Memory region for displaying bitmapped graphics
pub trait Framebuffer {
    /// Returns some information including dimensions, pixel format, etc.
    fn info(&self) -> FramebufferInfo;

    /// Returns a mutable slice directly into framebuffer memory.
    fn get_mut(&mut self) -> &mut [u8];

    /// Draws a rectangle directly to the framebuffer with top left corner at coords (x,y).
    /// This is the preferred way to draw on framebuffers, since `GfxRectangle`s have a consistent
    /// format, and the implementor can take advantage of faster algorithms, if available. A default
    /// implementation is provided, as long as the pixel channels are byte-wide and byte-aligned.
    ///
    /// # Requirements
    /// The following rules must be followed by anything implementing this trait:
    ///
    /// ## Out-of-bounds
    /// Out of bounds pixels (for example, calling with `coords = (-10,5)`) should not be drawn and
    /// quietly ignored.
    ///
    /// ## Alpha channel transparency
    /// If the alpha channel of a pixel in `rect` is exactly zero, then do not draw that pixel.
    /// Implementations may implement blending if they want, but it is not necessary.
    fn blit(&mut self, rect: &GfxRectangle, coords: (i32, i32)) {
        let info = self.info();
        let buf = self.get_mut();

        let xstart = coords.0.max(0); // xstart >= c.0
        let ystart = coords.1.max(0);
        let xoff = (xstart - coords.0) as u32; // xstart - c.0 >= 0
        let yoff = (ystart - coords.1) as u32;
        let xend = (coords.0 + rect.width as i32).min(info.width as i32);
        let yend = (coords.1 + rect.height as i32).min(info.height as i32);

        // Stop early if the drawn rectangle is completely off-screen
        if xend <= xstart || yend <= ystart {
            return;
        }

        // Compute pixel format information
        let fmt = info.format;
        if fmt.red_width_bits != 8 || fmt.blue_width_bits != 8 || fmt.green_width_bits != 8 {
            log::warn!("Framebuffer pixel format has non byte-sized channels");
            return;
        }
        if fmt.red_shift_bits % 8 != 0
            || fmt.blue_shift_bits % 8 != 0
            || fmt.green_shift_bits % 8 != 0
        {
            log::warn!("Framebuffer pixel format has non byte-aligned channels");
            return;
        }
        let r_off = (fmt.red_shift_bits / 8) as usize;
        let g_off = (fmt.green_shift_bits / 8) as usize;
        let b_off = (fmt.blue_shift_bits / 8) as usize;

        for fby in ystart..yend {
            let ry = (fby - ystart) as u32 + yoff;
            for fbx in xstart..xend {
                let rx = (fbx - xstart) as u32 + xoff;
                let pix = rect[(rx, ry)];
                if pix.a == 0 {
                    continue;
                }
                let fb_off = fby as usize * info.stride + fbx as usize * info.bytes_per_pixel;
                buf[fb_off + r_off] = pix.r;
                buf[fb_off + g_off] = pix.g;
                buf[fb_off + b_off] = pix.b;
            }
        }
    }
}

/// A no-op framebuffer
pub struct NullFramebuffer;
impl Framebuffer for NullFramebuffer {
    fn info(&self) -> FramebufferInfo {
        FramebufferInfo {
            format: PixelFormat::RGBA,
            bytes_per_pixel: 4,
            width: 0,
            height: 0,
            stride: 0,
            buffer_len: 0,
        }
    }

    fn get_mut(&mut self) -> &mut [u8] {
        &mut []
    }

    fn blit(&mut self, _rect: &GfxRectangle, _coords: (i32, i32)) {}
}

impl<F: Framebuffer + ?Sized> Framebuffer for Box<F> {
    fn info(&self) -> FramebufferInfo {
        self.as_ref().info()
    }
    fn get_mut(&mut self) -> &mut [u8] {
        self.as_mut().get_mut()
    }
    fn blit(&mut self, rect: &GfxRectangle, coords: (i32, i32)) {
        self.as_mut().blit(rect, coords)
    }
}

impl<F: Framebuffer + ?Sized> Framebuffer for &mut F {
    fn info(&self) -> FramebufferInfo {
        (**self).info()
    }
    fn get_mut(&mut self) -> &mut [u8] {
        (**self).get_mut()
    }
    fn blit(&mut self, rect: &GfxRectangle, coords: (i32, i32)) {
        (**self).blit(rect, coords)
    }
}

#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    pub format: PixelFormat,
    pub bytes_per_pixel: usize,

    // Horizontal length in pixels
    pub width: u32,
    // Number of rows
    pub height: u32,

    // Bytes per row
    pub stride: usize,
    // Total length in bytes
    pub buffer_len: usize,
}

/// Describes RGB channel layout.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PixelFormat {
    pub red_shift_bits: u8,
    pub red_width_bits: u8,
    pub green_shift_bits: u8,
    pub green_width_bits: u8,
    pub blue_shift_bits: u8,
    pub blue_width_bits: u8,
}

impl PixelFormat {
    /// The format used by [`Pixel`]
    pub const RGBA: Self = PixelFormat {
        red_shift_bits: 0,
        green_shift_bits: 8,
        blue_shift_bits: 16,
        red_width_bits: 8,
        green_width_bits: 8,
        blue_width_bits: 8,
    };
}

/// A standardized graphics rectangle. Backed by a `Vec<Pixel>`.
pub struct GfxRectangle {
    buf: Vec<Pixel>,
    width: u32,
    height: u32,
}

impl GfxRectangle {
    /// Create a blank, completely transparent `GfxRectangle`
    pub fn blank(width: u32, height: u32) -> Self {
        let buf = vec![Pixel::BLANK; (width * height) as usize];
        GfxRectangle { buf, width, height }
    }

    /// Create a rectangle initialized with pixels computed from a function.
    pub fn with(width: u32, height: u32, mut f: impl FnMut(u32, u32) -> Pixel) -> Self {
        let mut s = Self::blank(width, height);
        for y in 0..height {
            for x in 0..width {
                s[(x, y)] = f(x, y);
            }
        }
        s
    }

    /// Get the pixel at coords (x,y). `None` if (x,y) is out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Option<&Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(&self.buf[(y * self.width + x) as usize])
    }

    /// Mutably borrow the pixel at coords (x,y). `None` if (x,y) is out of bounds.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(&mut self.buf[(y * self.width + x) as usize])
    }
}

impl Index<(u32, u32)> for GfxRectangle {
    type Output = Pixel;
    fn index(&self, index: (u32, u32)) -> &Self::Output {
        self.get(index.0, index.1).expect("Index out of bounds")
    }
}

impl IndexMut<(u32, u32)> for GfxRectangle {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        self.get_mut(index.0, index.1).expect("Index out of bounds")
    }
}

/// A 32-bit RGBA pixel.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    const BLANK: Self = Pixel::new_rgba(0, 0, 0, 255);
    // const BLACK: Self = Pixel::new_rgb(0, 0, 0);
    // const WHITE: Self = Pixel::new_rgb(255, 255, 255);

    #[inline]
    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Pixel { r, g, b, a }
    }

    #[inline]
    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Pixel { r, g, b, a: 255 }
    }

    /// Intended for ease of use with hex color codes. (0xRRGGBBAA)
    #[inline]
    pub const fn from_u32_rgba(x: u32) -> Self {
        let xs = x.to_be_bytes();
        Pixel {
            r: xs[0],
            g: xs[1],
            b: xs[2],
            a: xs[3],
        }
    }

    /// Intended for ease of use with hex color codes. (0x00RRGGBB)
    #[inline]
    pub const fn from_u32_rgb(x: u32) -> Self {
        let xs = x.to_be_bytes();
        Pixel {
            r: xs[1],
            g: xs[2],
            b: xs[3],
            a: 255,
        }
    }
}

impl Display for Pixel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "#{:02X}{:02X}{:02X}{:02X}",
            self.r, self.g, self.b, self.a
        )
    }
}
