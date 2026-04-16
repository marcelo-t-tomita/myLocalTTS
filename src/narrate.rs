use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use whatlang::{detect, Lang};

/// Configuration for Piper TTS
pub struct NarratorConfig {
    pub piper_path: PathBuf,
    pub models: HashMap<String, PathBuf>, // language code -> model path
    pub default_model: PathBuf,
    pub speed: f32,
}

impl NarratorConfig {
    /// Load TTS configuration from environment variables or config file
    /// Priority: Environment variables > config file > defaults
    ///
    /// Supports language-specific models with PIPER_MODEL_XX format:
    /// - PIPER_MODEL_EN for English
    /// - PIPER_MODEL_PT for Portuguese
    /// - PIPER_MODEL (or PIPER_MODEL_DEFAULT) as fallback
    pub fn load() -> Result<Self> {
        let current_dir = env::current_dir()?;

        // Try to load from config file first
        let config_path = current_dir.join("tts_config.txt");
        let mut piper_path: Option<PathBuf> = None;
        let mut models: HashMap<String, PathBuf> = HashMap::new();
        let mut default_model: Option<PathBuf> = None;
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

                    if key == "PIPER_PATH" {
                        piper_path = Some(PathBuf::from(value));
                    } else if key == "PIPER_MODEL" || key == "PIPER_MODEL_DEFAULT" {
                        default_model = Some(PathBuf::from(value));
                    } else if key.starts_with("PIPER_MODEL_") {
                        // Extract language code (e.g., "EN" from "PIPER_MODEL_EN")
                        let lang_code = key.strip_prefix("PIPER_MODEL_").unwrap().to_lowercase();
                        if lang_code != "default" {
                            models.insert(lang_code, PathBuf::from(value));
                        }
                    } else if key == "SPEED" {
                        speed = value.parse().unwrap_or(1.0);
                    }
                }
            }
        }

        // Environment variables override config file
        if let Ok(path) = env::var("PIPER_PATH") {
            piper_path = Some(PathBuf::from(path));
        }
        if let Ok(path) = env::var("PIPER_MODEL") {
            default_model = Some(PathBuf::from(path));
        }

        // Default paths if not configured
        let piper_path = piper_path.unwrap_or_else(|| current_dir.join("piper.exe"));
        let default_model = default_model.unwrap_or_else(|| current_dir.join("piper-model.onnx"));

        // Validate piper executable
        if !piper_path.exists() {
            return Err(anyhow!(
                "Piper executable not found at '{}'. Please download Piper from https://github.com/OHF-Voice/piper1-gpl/releases and set PIPER_PATH environment variable or add PIPER_PATH to tts_config.txt",
                piper_path.display()
            ));
        }

        // Validate default model
        if !default_model.exists() {
            return Err(anyhow!(
                "Piper model not found at '{}'. Please download a model from https://huggingface.co/rhasspy/piper-voices and set PIPER_MODEL environment variable or add PIPER_MODEL to tts_config.txt",
                default_model.display()
            ));
        }

        // Validate language-specific models and remove invalid ones
        models.retain(|lang, path| {
            if path.exists() {
                true
            } else {
                eprintln!(
                    "WARNING: Piper model for '{}' not found at '{}', will use default",
                    lang,
                    path.display()
                );
                false
            }
        });

        // Log detected models
        if !models.is_empty() {
            println!("Language-specific TTS models loaded:");
            for (lang, path) in &models {
                println!("  {} -> {}", lang.to_uppercase(), path.display());
            }
            println!("  DEFAULT -> {}", default_model.display());
        }

        Ok(Self {
            piper_path,
            models,
            default_model,
            speed,
        })
    }

    /// Get the appropriate model path for the given text
    pub fn get_model_for_text(&self, text: &str) -> &PathBuf {
        if let Some(info) = detect(text) {
            let lang_code = match info.lang() {
                Lang::Eng => "en",
                Lang::Por => "pt",
                Lang::Spa => "es",
                Lang::Fra => "fr",
                Lang::Deu => "de",
                Lang::Ita => "it",
                Lang::Nld => "nl",
                Lang::Rus => "ru",
                Lang::Jpn => "ja",
                Lang::Cmn => "zh",
                Lang::Kor => "ko",
                Lang::Ara => "ar",
                Lang::Hin => "hi",
                Lang::Tur => "tr",
                Lang::Pol => "pl",
                Lang::Ukr => "uk",
                Lang::Ces => "cs",
                Lang::Ron => "ro",
                Lang::Hun => "hu",
                Lang::Ell => "el",
                Lang::Swe => "sv",
                Lang::Dan => "da",
                Lang::Fin => "fi",
                Lang::Nob => "no", // Norwegian Bokmål
                _ => "default",
            };

            // Check confidence - only use detected language if confident enough
            if info.is_reliable() {
                if let Some(model) = self.models.get(lang_code) {
                    println!(
                        "[TTS] Detected language: {} (confidence: {:.0}%)",
                        lang_code.to_uppercase(),
                        info.confidence() * 100.0
                    );
                    return model;
                }
            } else {
                println!("[TTS] Language detection failed");
                println!("[TTS] Confidence: {:.0}%", info.confidence() * 100.0);
            }
        }

        &self.default_model
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

            // Select model based on detected language
            let model_path = self.config.get_model_for_text(text);

            // Run Piper to generate WAV file
            // --length-scale: <1.0 = faster, >1.0 = slower (default 1.0)
            let piper_result = Command::new(&self.config.piper_path)
                .arg("--model")
                .arg(model_path)
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
                    // Escape single quotes to prevent PowerShell command injection
                    let safe_path = temp_audio.display().to_string().replace("'", "''");
                    let player = Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-WindowStyle",
                            "Hidden",
                            "-Command",
                            &format!(
                                "(New-Object Media.SoundPlayer '{}').PlaySync()",
                                safe_path
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

            let model_path = self.config.get_model_for_text(text);

            let mut piper = Command::new(&self.config.piper_path)
                .arg("--model")
                .arg(model_path)
                .arg("--length-scale")
                .arg(self.config.speed.to_string())
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
