mod audio;
mod transcribe;
mod clipboard;

use audio::{AudioRecorder, list_input_devices, get_device_by_index};
use transcribe::Transcriber;
use clipboard::ClipboardManager;
use inputbot::KeybdKey;
use std::time::Duration;
use std::io::{self, Write};
use anyhow::Result;

fn select_microphone() -> Result<usize> {
    let devices = list_input_devices()?;

    if devices.is_empty() {
        return Err(anyhow::anyhow!("No input devices found"));
    }

    println!("\nAvailable microphones:");
    for (i, name) in devices.iter().enumerate() {
        println!("  [{}] {}", i + 1, name);
    }

    loop {
        print!("\nSelect microphone (1-{}): ", devices.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(num) = input.trim().parse::<usize>() {
            if num >= 1 && num <= devices.len() {
                println!("Selected: {}", devices[num - 1]);
                return Ok(num - 1);
            }
        }
        println!("Invalid selection. Please try again.");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Local TTS Tool...");

    // 1. Initialize Components
    let path_to_model = "ggml-large-v3-turbo.bin"; // Best model with CUDA acceleration
    if !std::path::Path::new(path_to_model).exists() {
        eprintln!("ERROR: Model file '{}' not found!", path_to_model);
        eprintln!("Please download a ggml model (e.g. from https://huggingface.co/ggerganov/whisper.cpp) and place it in the project root.");
        return Ok(());
    }

    // Select microphone
    let device_index = select_microphone()?;
    let device = get_device_by_index(device_index)?;

    let mut recorder = AudioRecorder::new();
    recorder.set_device(device);
    let transcriber = match Transcriber::new(path_to_model) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialize Whisper: {}", e);
            return Ok(());
        }
    };
    let mut clipboard_mgr = ClipboardManager::new()?;

    println!("Listening for F9 (Hold to record, release to transcribe)...");

    let mut was_pressed = false;

    // Event Loop - poll F9 key state
    loop {
        let is_pressed = KeybdKey::F9Key.is_pressed();

        if is_pressed && !was_pressed {
            // Key just pressed - start recording
            println!("Recording started...");
            if let Err(e) = recorder.start() {
                eprintln!("Failed to start recording: {}", e);
            }
        } else if !is_pressed && was_pressed {
            // Key just released - stop and transcribe
            println!("Recording stopped. Transcribing...");
            match recorder.stop() {
                Ok(audio_data) => {
                    if audio_data.is_empty() {
                        println!("Audio buffer empty, ignoring.");
                        was_pressed = is_pressed;
                        continue;
                    }

                    println!("Captured {} samples.", audio_data.len());

                    let temp_filename = "temp_input.wav";
                    if let Err(e) = recorder.save_to_file(&audio_data, temp_filename) {
                        eprintln!("Failed to save WAV file: {}", e);
                        was_pressed = is_pressed;
                        continue;
                    }

                    match transcriber.transcribe(temp_filename) {
                        Ok(text) => {
                            println!("Transcribed: '{}'", text);
                            if !text.is_empty() {
                                if let Err(e) = clipboard_mgr.paste_text(&text) {
                                    eprintln!("Failed to paste: {}", e);
                                }
                            }
                        },
                        Err(e) => eprintln!("Transcription failed: {}", e),
                    }
                },
                Err(e) => eprintln!("Failed to stop recording: {}", e),
            }
        }

        was_pressed = is_pressed;
        std::thread::sleep(Duration::from_millis(20));
    }
}
