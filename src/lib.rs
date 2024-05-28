use std::io::{self, Stdout, StdoutLock, Write};

use crossterm::{
    cursor::{Hide, Show},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetSize,
    },
    QueueableCommand,
};
use fast_image_resize::{images::TypedImage, IntoImageView, Resizer};
use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, ImageDecoder, Luma};
use rayon::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Other error: {0}")]
    Other(String),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Screen<W: io::Write> {
    size: (u16, u16),
    // It's a better idea to have a 1D array for simplicity, especially since we have x and y
    writer: W,

    // 1D array is the play here
    buf: Vec<u8>, // We only deal with ASCII for now
}
pub const DENSITY: [u8; 66] = [
    b' ', b'`', b'^', b'"', b',', b':', b';', b'I', b'l', b'!', b'i', b'~', b'+', b'_', b'-', b'?',
    b']', b'[', b'}', b'{', b'1', b')', b'(', b'|', b'\\', b'/', b't', b'f', b'j', b'r', b'x',
    b'n', b'u', b'v', b'c', b'z', b'X', b'Y', b'U', b'J', b'C', b'L', b'Q', b'0', b'O', b'Z', b'm',
    b'w', b'q', b'p', b'd', b'b', b'k', b'h', b'a', b'o', b'*', b'#', b'M', b'W', b'&', b'8', b'%',
    b'B', b'@', b'$',
];
pub type TerminalScreen = Screen<StdoutLock<'static>>;

impl TerminalScreen {
    pub fn init() -> Result<Self> {
        let size = terminal::size()?;
        let mut stdout = io::stdout().lock();
        stdout
            // .queue(terminal::Clear(ClearType::All))?
            .queue(EnterAlternateScreen)?
            .queue(Hide)?
            .queue(SetSize(size.0, size.1))?;
        stdout.flush()?;

        enable_raw_mode()?; // Not sure if this works - we probably want to operate on stdout
        let size = terminal::size()?;
        Self::init_with_size_and_writer(size, stdout)
    }
    pub fn close(mut self) -> Result<()> {
        self.writer.queue(Show)?.queue(LeaveAlternateScreen)?;
        // .queue(Clear(ClearType::All))?;
        self.writer.flush()?;
        disable_raw_mode()?;
        Ok(())
    }
}

impl<W> Screen<W>
where
    W: io::Write,
{
    // Full pixels
    pub fn init_with_size_and_writer(size: (u16, u16), writer: W) -> Result<Self> {
        let buf = vec![b' '; size.0 as usize * size.1 as usize];

        Ok(Screen { size, writer, buf })
    }

    pub fn draw_square_byte(&mut self, coords: (u16, u16), size: (u16, u16), char: u8) {
        let (width, height) = self.size;
        let (x, y) = coords;
        let (w, h) = size;
        for row in 0..h {
            for col in 0..w {
                let idx = ((y + row) * width + x + col) as usize;
                self.buf[idx] = char;
            }
        }
    }
    // TODO: Speed ts up?
    pub fn draw_square_bytes(&mut self, coords: (u16, u16), size: (u16, u16), buffer: &[u8]) {
        let (width, height) = self.size;
        let (x, y) = coords;
        let (w, h) = size;
        for j in 0..h {
            for i in 0..w {
                let idx = ((y + j) * width + x + i) as usize;
                if let Some(byte) = buffer.get((j * w + i) as usize) {
                    self.buf[idx] = *byte;
                }
            }
        }
    }

    pub fn buffer(&self) -> &[u8] {
        self.buf.as_slice()
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }
    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn finish(&mut self) -> Result<()> {
        Ok(self.writer.write_all(self.buf.as_slice())?)
    }
}

pub trait Sprite {
    fn to_ascii(&self, dst_size: (u16, u16)) -> Result<Vec<u8>>;
}

pub struct ImageSprite {
    img: DynamicImage, //FIXME: Probably better to use an ImageBuffer here
}

impl ImageSprite {
    pub fn from_file(path: &str) -> Result<Self> {
        let img = image::open(path)?.grayscale();
        Ok(Self::new(img))
    }
    pub fn new(img: DynamicImage) -> Self {
        Self { img }
    }
}

impl Sprite for ImageSprite {
    // fn to_ascii(&self) -> Vec<u8> {
    //     let size = (self.img.width() as u16, self.img.height() as u16);
    //     self.to_ascii_with_size(size)
    // }
    // FIXME: This is terrible written - Figure out how to use ImageBuffer and minimal copying
    fn to_ascii(&self, dst_size: (u16, u16)) -> Result<Vec<u8>> {
        let mut resizer = Resizer::new();
        let mut dst_image =
            DynamicImage::new(dst_size.0 as u32, dst_size.1 as u32, self.img.color());
        resizer.resize(&self.img, &mut dst_image, None).unwrap();

        // resizer
        //     .resize_typed(&self.img, &mut dst_image, None)
        //     .unwrap();
        Ok(dst_image
            .into_luma16()
            .par_pixels()
            .map(|px| {
                let [brightness] = px.0;
                let idx =
                    ((brightness as f32 / u16::MAX as f32) * ((DENSITY.len() - 1) as f32)) as usize;
                // let idx = (brightness as f32 * (255.0 / DENSITY.len() as f32)) as usize;
                DENSITY[idx]
            })
            .collect())
    }
}
