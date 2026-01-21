use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// Configuration for Piper TTS
pub struct NarratorConfig {
    pub piper_path: PathBuf,
    pub model_path: PathBuf,
    pub speed: f32,
}

impl NarratorConfig {
    /// Load TTS configuration from environment variables or config file
    /// Priority: Environment variables > config file > defaults
    pub fn load() -> Result<Self> {
        let current_dir = env::current_dir()?;

        // Try to load from config file first
        let config_path = current_dir.join("tts_config.txt");
        let mut piper_path: Option<PathBuf> = None;
        let mut model_path: Option<PathBuf> = None;
        let mut speed: f32 = 1.0;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        "PIPER_PATH" => piper_path = Some(PathBuf::from(value)),
                        "PIPER_MODEL" => model_path = Some(PathBuf::from(value)),
                        "SPEED" => speed = value.parse().unwrap_or(1.0),
                        _ => {}
                    }
                }
            }
        }

        // Environment variables override config file
        if let Ok(path) = env::var("PIPER_PATH") {
            piper_path = Some(PathBuf::from(path));
        }
        if let Ok(path) = env::var("PIPER_MODEL") {
            model_path = Some(PathBuf::from(path));
        }

        // Default paths if not configured
        let piper_path = piper_path.unwrap_or_else(|| current_dir.join("piper.exe"));
        let model_path = model_path.unwrap_or_else(|| current_dir.join("piper-model.onnx"));

        // Validate paths
        if !piper_path.exists() {
            return Err(anyhow!(
                "Piper executable not found at '{}'. Please download Piper from https://github.com/OHF-Voice/piper1-gpl/releases and set PIPER_PATH environment variable or add PIPER_PATH to tts_config.txt",
                piper_path.display()
            ));
        }

        if !model_path.exists() {
            return Err(anyhow!(
                "Piper model not found at '{}'. Please download a model from https://huggingface.co/rhasspy/piper-voices and set PIPER_MODEL environment variable or add PIPER_MODEL to tts_config.txt",
                model_path.display()
            ));
        }

        Ok(Self {
            piper_path,
            model_path,
            speed,
        })
    }
}

/// Manages TTS playback with cancellation support
pub struct Narrator {
    config: NarratorConfig,
    current_process: Arc<Mutex<Option<Child>>>,
}

impl Narrator {
    pub fn new(config: NarratorConfig) -> Self {
        Self {
            config,
            current_process: Arc::new(Mutex::new(None)),
        }
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        if let Ok(mut guard) = self.current_process.lock() {
            if let Some(ref mut child) = *guard {
                // Check if process is still running
                match child.try_wait() {
                    Ok(None) => return true, // Still running
                    Ok(Some(_)) => {
                        // Process finished, clean up
                        *guard = None;
                        return false;
                    }
                    Err(_) => {
                        *guard = None;
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Stop current playback if any
    pub fn stop(&self) -> Result<()> {
        if let Ok(mut guard) = self.current_process.lock() {
            if let Some(ref mut child) = guard.take() {
                // Kill the process tree on Windows
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    // Use taskkill to kill the process and its children
                    let _ = Command::new("taskkill")
                        .args(["/F", "/T", "/PID", &child.id().to_string()])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .output();
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let _ = child.kill();
                }

                let _ = child.wait();
            }
        }
        Ok(())
    }

    /// Speak the given text using Piper TTS
    /// This spawns a non-blocking process that pipes Piper output to an audio player
    pub fn speak(&self, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Err(anyhow!("No text to speak"));
        }

        // Stop any current playback first
        self.stop()?;

        #[cfg(target_os = "windows")]
        {
            use std::io::Write;
            use std::os::windows::process::CommandExt;

            let temp_audio = env::temp_dir().join("tts_output.wav");

            // Debug: print the command being run
            //println!(
            //    self.config.piper_path.display(),
            //    self.config.model_path.display(),
            //    temp_audio.display()
            //);

            // Run Piper to generate WAV file
            // --length-scale: <1.0 = faster, >1.0 = slower (default 1.0)
            let piper_result = Command::new(&self.config.piper_path)
                .arg("--model")
                .arg(&self.config.model_path)
                .arg("--length-scale")
                .arg(self.config.speed.to_string())
                .arg("--output_file")
                .arg(&temp_audio)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .spawn();

            match piper_result {
                Ok(mut child) => {
                    // Write text to Piper's stdin
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    // Drop stdin to signal EOF
                    drop(child.stdin.take());

                    // Wait for Piper to finish generating audio
                    let output = child.wait_with_output()?;
                    if !output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(anyhow!(
                            "Piper failed (exit code {:?}): stdout='{}' stderr='{}'",
                            output.status.code(),
                            stdout.trim(),
                            stderr.trim()
                        ));
                    }

                    // Play the audio file using PowerShell (non-blocking)
                    let player = Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-WindowStyle",
                            "Hidden",
                            "-Command",
                            &format!(
                                "(New-Object Media.SoundPlayer '{}').PlaySync()",
                                temp_audio.display()
                            ),
                        ])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .spawn()?;

                    // Store the player process for cancellation
                    if let Ok(mut guard) = self.current_process.lock() {
                        *guard = Some(player);
                    }
                }
                Err(e) => {
                    return Err(anyhow!("Failed to start Piper: {}", e));
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            use std::io::Write;

            let player_cmd = if cfg!(target_os = "macos") {
                "afplay"
            } else {
                "aplay"
            };

            let temp_audio = env::temp_dir().join("tts_output.wav");

            let mut piper = Command::new(&self.config.piper_path)
                .arg("--model")
                .arg(&self.config.model_path)
                .arg("--output_file")
                .arg(&temp_audio)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;

            if let Some(ref mut stdin) = piper.stdin {
                let _ = stdin.write_all(text.as_bytes());
            }
            drop(piper.stdin.take());
            piper.wait()?;

            let player = Command::new(player_cmd).arg(&temp_audio).spawn()?;

            if let Ok(mut guard) = self.current_process.lock() {
                *guard = Some(player);
            }
        }

        Ok(())
    }
}
