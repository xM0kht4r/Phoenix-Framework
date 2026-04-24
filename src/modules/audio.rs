use anyhow::{Result, Context};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait},Stream, SupportedStreamConfig};
use hound::{WavSpec, WavWriter};
use std::{env, sync::{Arc, Mutex}, time::Duration};
use crate::modules::random;

/// Captures audio for the specified duration and saves it to a WAV
pub async fn record(seconds: &str) -> Result<String> {
    let (audio_data, _stream) = init_audio_capture()?;

    let num:u64 = seconds.parse()?;
    std::thread::sleep(Duration::from_secs(num));
    let path = save_wav(&audio_data)?;

    Ok(path)
}

fn init_audio_capture() -> Result<(Arc<Mutex<Vec<f32>>>, Stream)> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .context("No input device available")?;

    let config: SupportedStreamConfig = input_device
        .default_input_config()
        .context("Failed to get default input config")?;

    let audio_data = Arc::new(Mutex::new(Vec::new()));
    let audio_data_clone = Arc::clone(&audio_data);

    let stream = input_device.build_input_stream(
        &config.config(),
        move |data: &[f32], _| audio_data_clone.lock().unwrap().extend_from_slice(data),
        |_| {}, // Do nothing with errors
        None,
    ).context("Failed to build input stream")?;

    stream.play().context("Failed to start stream")?;
    Ok((audio_data, stream))
}

fn save_wav(audio_data: &Arc<Mutex<Vec<f32>>>) -> Result<String> {
    
    let name = format!("{}.wav", random::random_name());
    let temp_dir = env::temp_dir();
    let wav_path = temp_dir.join(name);

    let config = cpal::default_host()
        .default_input_device()
        .context("No input device available")?
        .default_input_config()
        .context("Failed to get default input config")?;

    let spec = WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(&wav_path, spec).context("Failed to create WAV file")?;
    let audio_data = audio_data.lock().unwrap();
    for sample in audio_data.iter() {
        writer
            .write_sample((*sample * i16::MAX as f32) as i16)
            .context("Failed to write sample to WAV file")?;
    }

    writer.finalize().context("Failed to finalize WAV file")?;
    Ok(wav_path.to_string_lossy().to_string())
}