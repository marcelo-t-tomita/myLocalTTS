use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct Transcriber {
    executable_path: PathBuf,
    model_path: PathBuf,
}

impl Transcriber {
    pub fn new(model_path: &str) -> Result<Self> {
        let current_dir = env::current_dir()?;

        // We look for 'whisper-cli.exe', 'whisper.exe', or 'main.exe' (deprecated)
        let possible_names = ["whisper-cli.exe", "whisper.exe", "main.exe"];
        let mut executable_path: Option<PathBuf> = None;

        for name in possible_names.iter() {
            let full_path = current_dir.join(name);
            if full_path.exists() {
                executable_path = Some(full_path);
                break;
            }
        }

        let executable_path = executable_path.ok_or_else(|| {
            anyhow!("Whisper executable not found. Please download 'whisper-cli.exe' from whisper.cpp releases and place it in the project root.")
        })?;

        let model_full_path = current_dir.join(model_path);

        Ok(Self {
            executable_path,
            model_path: model_full_path,
        })
    }

    pub fn transcribe(&self, audio_filename: &str) -> Result<String> {
        let current_dir = env::current_dir()?;
        let audio_path = current_dir.join(audio_filename);

        let output = Command::new(&self.executable_path)
            .arg("-m")
            .arg(&self.model_path)
            .arg("-f")
            .arg(&audio_path)
            .arg("--output-txt")
            .arg("-nt") // No timestamps in output
            .arg("-l")
            .arg("auto") // Auto-detect language
            .arg("--prompt")
            .arg("Multilingual transcription. Transcrição multilíngue. English and Portuguese text. Texto em inglês e português.")
            .output()
            .map_err(|e| anyhow!("Failed to execute whisper process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("Whisper stdout: {}", stdout);
            eprintln!("Whisper stderr: {}", stderr);
            return Err(anyhow!(
                "Whisper process execution failed (exit code: {:?})",
                output.status.code()
            ));
        }

        let raw_output = String::from_utf8_lossy(&output.stdout).to_string();

        // Cleanup common artifacts
        let clean_text = raw_output
            .trim()
            .replace("[BLANK_AUDIO]", "")
            .replace("[MÚSICA]", "")
            .replace("[MÚSICA DE FUNDO]", "")
            .trim()
            .to_string();

        Ok(clean_text)
    }
}
