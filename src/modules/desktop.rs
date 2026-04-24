use anyhow::{Result, Context, bail};
use scrap::{Capturer, Display};
use image::codecs::jpeg::JpegEncoder;
use tokio::sync::mpsc;
use bytes::{Bytes, BufMut, BytesMut};
use async_stream;
use axum::{response::Response, body::Body};

pub fn start(tx: mpsc::Sender<Bytes>) -> Result<()> {

	let display = Display::primary()?;
	let (height, width) = (display.height(), display.width());
	let mut rgb_buffer  = vec![0u8; width * height * 3];
	let mut jpeg_buffer = Vec::with_capacity(width * height * 2);

	let mut capturer = Capturer::new(display)?;

	loop {
		let frame = match capturer.frame() {
			Ok(frame) => frame,
			Err(e) => {
				std::thread::sleep(std::time::Duration::from_millis(10));
				continue;
				}
			};
			// Ensure frame has data
            if frame.len() < width * height * 4 {
                continue;
            }
		    // Convert BGRA to RGB
            rgb_buffer.clear();
			rgb_buffer.resize(width * height * 3, 0);

            for (i, pixel) in frame.chunks_exact(4).enumerate() {
                if i * 3 + 2 < rgb_buffer.len() {
                    rgb_buffer[i * 3] = pixel[2];     // R
                    rgb_buffer[i * 3 + 1] = pixel[1]; // G
                    rgb_buffer[i * 3 + 2] = pixel[0]; // B
                }
            }


			// Encode RGB buffer to JPEG
            jpeg_buffer.clear();
            let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_buffer, 85);
            if let Err(e) = encoder.encode(&rgb_buffer, width as u32, height as u32, image::ExtendedColorType::Rgb8) {
            	continue;
            }

            if jpeg_buffer.is_empty() {
                continue;
            }

            let jpeg_bytes = Bytes::copy_from_slice(&jpeg_buffer);

            if tx.blocking_send(jpeg_bytes).is_err() {
            	bail!("Stream ended!");
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
	}


	Ok(())
}

pub async fn stream_handler() -> Response {
	
	let (tx, mut rx) = mpsc::channel::<Bytes>(10);
	std::thread::spawn(move || {start(tx)});
	
    // Create MJPEG stream from received JPEGs
    let stream = async_stream::stream! {
        while let Some(jpeg) = rx.recv().await {
            let mut frame = BytesMut::new();
            frame.extend_from_slice(b"--frame\r\n");
            frame.extend_from_slice(b"Content-Type: image/jpeg\r\n");
            frame.extend_from_slice(b"Content-Length: ");
            frame.extend_from_slice(jpeg.len().to_string().as_bytes());
            frame.extend_from_slice(b"\r\n\r\n");
            frame.extend_from_slice(&jpeg);
            frame.extend_from_slice(b"\r\n");
            
            yield Ok::<_, std::convert::Infallible>(frame.freeze());
        }
    };
    
    // Return the HTTP response
    Response::builder()
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(Body::from_stream(stream))
        .expect("Failed to create a reponse! Error: {}")

}

