use atengine::Result;
use atengine::Screen;
use atengine::TerminalScreen;
use std::thread;
use std::time::Duration;
fn main() -> Result<()> {
    let mut screen = TerminalScreen::init()?;
    screen.buffer_mut().iter_mut().for_each(|cell| *cell = b'X');
    screen.draw()?;
    thread::sleep(Duration::from_secs(2));
    screen.close()?;
    Ok(())
}
