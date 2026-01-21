mod audio;
mod clipboard;
mod narrate;
mod transcribe;

use anyhow::Result;
use audio::{get_device_by_index, list_input_devices, AudioRecorder};
use clipboard::ClipboardManager;
use inputbot::KeybdKey;
use narrate::{Narrator, NarratorConfig};
use std::io::{self, Write};
use std::time::Duration;
use transcribe::Transcriber;

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

    // Initialize TTS narrator (optional - will warn if not configured)
    let narrator: Option<Narrator> = match NarratorConfig::load() {
        Ok(config) => {
            println!("TTS narrator initialized with Piper.");
            Some(Narrator::new(config))
        }
        Err(e) => {
            eprintln!("WARNING: TTS narrator not available: {}", e);
            eprintln!("F10 text-to-speech will be disabled.");
            None
        }
    };

    println!("\nHotkeys:");
    println!("  F9  - Hold to record, release to transcribe (Speech-to-Text)");
    if narrator.is_some() {
        println!("  F10 - Read selected text aloud (Text-to-Speech)");
        println!("        Press F10 again while playing to stop");
    }
    println!("\nListening...");

    let mut was_f9_pressed = false;
    let mut was_f10_pressed = false;

    // Event Loop - poll F9 and F10 key states
    loop {
        let is_f9_pressed = KeybdKey::F9Key.is_pressed();
        let is_f10_pressed = KeybdKey::F10Key.is_pressed();

        // F9 handling - Speech-to-Text
        if is_f9_pressed && !was_f9_pressed {
            // Key just pressed - start recording
            println!("Recording started...");
            if let Err(e) = recorder.start() {
                eprintln!("Failed to start recording: {}", e);
            }
        } else if !is_f9_pressed && was_f9_pressed {
            // Key just released - stop and transcribe
            println!("Recording stopped. Transcribing...");
            match recorder.stop() {
                Ok(audio_data) => {
                    if audio_data.is_empty() {
                        println!("Audio buffer empty, ignoring.");
                        was_f9_pressed = is_f9_pressed;
                        continue;
                    }

                    println!("Captured {} samples.", audio_data.len());

                    let temp_filename = "temp_input.wav";
                    if let Err(e) = recorder.save_to_file(&audio_data, temp_filename) {
                        eprintln!("Failed to save WAV file: {}", e);
                        was_f9_pressed = is_f9_pressed;
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
                        }
                        Err(e) => eprintln!("Transcription failed: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to stop recording: {}", e),
            }
        }

        // F10 handling - Text-to-Speech
        if is_f10_pressed && !was_f10_pressed {
            if let Some(ref narrator) = narrator {
                if narrator.is_playing() {
                    // Stop current playback
                    println!("Stopping TTS playback...");
                    if let Err(e) = narrator.stop() {
                        eprintln!("Failed to stop playback: {}", e);
                    }
                } else {
                    // Get selected text and speak it
                    match get_selected_text() {
                        Ok(text) => {
                            if text.trim().is_empty() {
                                println!("No text selected.");
                            } else {
                                println!("Speaking: '{}'", truncate_for_display(&text, 50));
                                if let Err(e) = narrator.speak(&text) {
                                    eprintln!("TTS failed: {}", e);
                                }
                            }
                        }
                        Err(e) => eprintln!("Failed to get selected text: {}", e),
                    }
                }
            } else {
                println!("TTS not available. Please configure Piper.");
            }
        }

        was_f9_pressed = is_f9_pressed;
        was_f10_pressed = is_f10_pressed;
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Get selected text by simulating Ctrl+C and reading from clipboard
fn get_selected_text() -> Result<String> {
    use arboard::Clipboard;
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    // Simulate Ctrl+C IMMEDIATELY to copy selected text before focus can change
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to init enigo: {:?}", e))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| anyhow::anyhow!("Enigo error: {:?}", e))?;
    enigo
        .key(Key::Unicode('c'), Direction::Click)
        .map_err(|e| anyhow::anyhow!("Enigo error: {:?}", e))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| anyhow::anyhow!("Enigo error: {:?}", e))?;

    // Wait for clipboard to be updated
    std::thread::sleep(Duration::from_millis(150));

    // Now read from clipboard
    let mut clipboard =
        Clipboard::new().map_err(|e| anyhow::anyhow!("Failed to access clipboard: {}", e))?;

    let selected_text = clipboard.get_text().unwrap_or_default();
    println!(
        "[DEBUG] Clipboard contains: '{}'",
        truncate_for_display(&selected_text, 50)
    );

    Ok(selected_text)
}

/// Truncate text for display purposes
fn truncate_for_display(text: &str, max_len: usize) -> String {
    let text = text.replace('\n', " ").replace('\r', "");
    if text.len() <= max_len {
        text
    } else {
        format!("{}...", &text[..max_len])
    }
}
