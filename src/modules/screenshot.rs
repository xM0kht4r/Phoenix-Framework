use anyhow::{Result, Context};
use scrap::{Capturer, Display};
use image::{ImageBuffer, Rgba, DynamicImage, Rgb};

use crate::modules::random;

pub async fn screenshot() -> Result<String> {
    let display = Display::primary()?;
    let (width, height) = (display.width() as u32, display.height() as u32);
    let mut capturer = Capturer::new(display)?;

    let frame = loop {
        match capturer.frame() {
            Ok(frame) => break frame,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e).context("Failed to capture frame")?,
        }
    };

    let temp_dir = std::env::temp_dir();
    let name = random::random_name();

    let img_file = format!("{}.png", name);
    let img_path = temp_dir.join(img_file);

    ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, frame.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?
        .save(&img_path)
        .context("Failed to save image!")?;

    Ok(img_path.to_string_lossy().to_string())
}
