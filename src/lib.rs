use std::io::{self, Stdout, StdoutLock, Write};

use crossterm::{
    cursor::{Hide, Show},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetSize,
    },
    QueueableCommand,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Screen<W: io::Write> {
    size: (u16, u16),
    // It's a better idea to have a 1D array for simplicity, especially since we have x and y
    writer: W,

    // 1D array is the play here
    buf: Vec<u8>, // We only deal with ASCII for now
}

pub type TerminalScreen = Screen<StdoutLock<'static>>;

impl TerminalScreen {
    pub const DENSITY: [char; 65] = [
        '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '~', '+', '_', '-', '?', ']', '[', '}',
        '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X',
        'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a',
        'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$',
    ];
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

    pub fn buffer(&self) -> &[u8] {
        self.buf.as_slice()
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    pub fn draw(&mut self) -> Result<()> {
        Ok(self.writer.write_all(self.buf.as_slice())?)
    }
}
