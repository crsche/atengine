use std::{thread, time::Duration};

use atengine::{ImageSprite, Result, Screen, Sprite, TerminalScreen};
fn main() -> Result<()> {
    let sprite = ImageSprite::from_file("src/default.png")?;
    let ascii_sprite = sprite.to_ascii((80, 40))?;
    let mut screen = TerminalScreen::init()?;
    screen.draw_square_byte((0, 0), screen.size(), b'X');
    screen.draw_square_bytes((0, 0), (80, 40), &ascii_sprite);
    screen.finish()?;
    thread::sleep(Duration::from_secs(2));
    screen.close()?;
    Ok(())
}
