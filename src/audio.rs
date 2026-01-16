use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Device;
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    device: Option<Device>,
    sample_rate: u32,
}

/// Returns a list of available input device names
pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices: Vec<String> = host
        .input_devices()?
        .filter_map(|d| d.name().ok())
        .collect();
    Ok(devices)
}

/// Gets a device by index from the input devices list
pub fn get_device_by_index(index: usize) -> Result<Device> {
    let host = cpal::default_host();
    let device = host
        .input_devices()?
        .nth(index)
        .ok_or_else(|| anyhow!("Device index {} not found", index))?;
    Ok(device)
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            device: None,
            sample_rate: 44100,
        }
    }

    pub fn set_device(&mut self, device: Device) {
        self.device = Some(device);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn start(&mut self) -> Result<()> {
        // Stop any existing stream first
        if let Some(stream) = self.stream.take() {
            drop(stream);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let device = self.device.as_ref()
            .ok_or_else(|| anyhow!("No input device selected"))?;

        let config: cpal::StreamConfig = device.default_input_config()?.into();
        self.sample_rate = config.sample_rate.0;

        // Clear buffer before starting new recording
        {
            let mut lock = self.buffer.lock().map_err(|_| anyhow!("Failed to lock buffer"))?;
            lock.clear();
        }

        let buffer_clone = self.buffer.clone();
        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                if let Ok(mut lock) = buffer_clone.lock() {
                    lock.extend_from_slice(data);
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<f32>> {
        // Stop the stream first
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        // Small delay to ensure stream callback has finished
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Get the data and clear the buffer
        let mut lock = self.buffer.lock().map_err(|_| anyhow!("Failed to lock buffer"))?;
        let data = std::mem::take(&mut *lock); // Takes the data and replaces with empty Vec
        Ok(data)
    }

    pub fn save_to_file(&self, audio_data: &[f32], filename: &str) -> Result<()> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(filename, spec)?;
        // Simple conversion from f32 (-1.0 to 1.0) to i16
        for &sample in audio_data {
            let amplitude = i16::MAX as f32;
            let val = (sample * amplitude).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer.write_sample(val)?;
        }
        writer.finalize()?;
        Ok(())
    }
}
