# myLocalTTS

A lightweight, offline speech-to-text and text-to-speech tool for Windows that runs locally using [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) and [Piper](https://github.com/OHF-Voice/piper1-gpl). Press a hotkey to record your voice, release to transcribe, and the text is automatically pasted into your active application. Or select text and press F10 to have it read aloud.

## Features

-   **Fully Offline** - No internet connection required, all processing happens locally
-   **GPU Accelerated** - Supports CUDA for fast transcription on NVIDIA GPUs
-   **Auto Language Detection** - Automatically detects the spoken language
-   **Speech-to-Text (F9)** - Hold F9 to record, release to transcribe and paste
-   **Text-to-Speech (F10)** - Select text and press F10 to read it aloud
-   **Microphone Selection** - Choose your preferred input device at startup
-   **Low Latency** - Text appears almost instantly after releasing the hotkey
-   **Cancellable TTS** - Press F10 again while audio is playing to stop

## Requirements

-   Windows 10/11
-   [Whisper.cpp](https://github.com/ggerganov/whisper.cpp/releases) executable (`whisper-cli.exe`) - for Speech-to-Text
-   A Whisper model file (e.g., `ggml-large-v3-turbo.bin`) - for Speech-to-Text
-   [Piper](https://github.com/OHF-Voice/piper1-gpl/releases) executable (`piper.exe`) - for Text-to-Speech (optional)
-   A Piper voice model (`.onnx` file) - for Text-to-Speech (optional)
-   For GPU acceleration: NVIDIA GPU with CUDA support

## Installation

### 1. Download the release

Download the latest release from the [Releases](https://github.com/marcelo-t-tomita/myLocalTTS/releases) page.

### 2. Download Whisper.cpp

> **Note:** The Whisper executable and DLLs are **not included** in this repository due to their size and licensing. You must download them separately.

1. Go to [whisper.cpp releases](https://github.com/ggerganov/whisper.cpp/releases)
2. Download the appropriate version:

    - **CPU only**: `whisper-bin-x64.zip`
    - **NVIDIA GPU (recommended)**: `whisper-cublas-12.4.0-bin-x64.zip`

    > **Tip:** The `whisper-cublas-12.4.0-bin-x64.zip` version has been tested and works best for CUDA GPU acceleration.

3. Extract the ZIP file
4. Copy the following files to your myLocalTTS folder:
    - `whisper-cli.exe` (required)
    - **All `.dll` files** (required - these include ggml, CUDA, and other dependencies)

Example DLLs you should see (varies by version):

```
ggml.dll
whisper.dll
cublas64_*.dll      (CUDA version only)
cublasLt64_*.dll    (CUDA version only)
cudart64_*.dll      (CUDA version only)
```

### 3. Download a Whisper model

> **Note:** Model files are **not included** in this repository due to their size (~75MB to ~3GB). You must download them separately.

Download a model from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp/tree/main):

| Model                     | Size    | Quality | Speed      |
| ------------------------- | ------- | ------- | ---------- |
| `ggml-tiny.bin`           | ~75 MB  | Basic   | Fastest    |
| `ggml-base.bin`           | ~147 MB | Good    | Fast       |
| `ggml-small.bin`          | ~466 MB | Better  | Medium     |
| `ggml-medium.bin`         | ~1.5 GB | Great   | Slower     |
| `ggml-large-v3-turbo.bin` | ~1.6 GB | Best    | Fast (GPU) |

For best results with a GPU, use `ggml-large-v3-turbo.bin`:

```powershell
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin" -OutFile "ggml-large-v3-turbo.bin"
```

### 4. Download Piper (Optional - for Text-to-Speech)

> **Note:** Piper is optional. If not configured, only Speech-to-Text (F9) will be available.

1. Go to [Piper releases](https://github.com/OHF-Voice/piper1-gpl/releases)
2. Download `piper_windows_amd64.zip`
3. Extract the ZIP file - it contains a `piper` folder with all required files
4. Copy the **entire `piper` folder** to your myLocalTTS directory (or rename/move it as needed)

**Important:** The `piper` folder must contain all of these files for Piper to work:

```
piper/
├── piper.exe                         (main executable)
├── espeak-ng.dll                     (required DLL)
├── espeak-ng-data/                   (required folder - phoneme data)
│   └── ... (many files inside)
├── onnxruntime.dll                   (required DLL)
├── onnxruntime_providers_shared.dll  (required DLL)
├── piper_phonemize.dll               (required DLL)
└── libtashkeel_model.ort             (required model file)
```

> **Warning:** If you only copy `piper.exe` without the DLLs and `espeak-ng-data` folder, you will get a "DLL not found" error (exit code -1073741515).

### 5. Download a Piper Voice Model (Optional)

Download a voice model from [Piper Voices](https://huggingface.co/rhasspy/piper-voices/tree/main):

1. Choose a language folder (e.g., `en/en_US` for US English, `pt/pt_BR` for Brazilian Portuguese)
2. Choose a speaker (e.g., `amy`, `lessac`)
3. Choose a quality level (`low`, `medium`, or `high`)
4. Download **both** files:
   - The `.onnx` model file (the voice model itself)
   - The `.onnx.json` config file (model configuration - **required**)
5. Place both files in your `piper` folder

Example for US English (Amy voice, medium quality):
```powershell
# Download to the piper folder
Invoke-WebRequest -Uri "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx" -OutFile "piper\piper-model.onnx"
Invoke-WebRequest -Uri "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx.json" -OutFile "piper\piper-model.onnx.json"
```

Example for Brazilian Portuguese (faber voice, medium quality):
```powershell
Invoke-WebRequest -Uri "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_BR/faber/medium/pt_BR-faber-medium.onnx" -OutFile "piper\piper-model.onnx"
Invoke-WebRequest -Uri "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_BR/faber/medium/pt_BR-faber-medium.onnx.json" -OutFile "piper\piper-model.onnx.json"
```

> **Note:** The `.onnx.json` file **must** have the same base name as the `.onnx` file. If you rename the model to `piper-model.onnx`, rename the config to `piper-model.onnx.json`.

### 6. Create the TTS Configuration File

Create a `tts_config.txt` file in your myLocalTTS folder to tell the application where to find Piper:

```
# Piper TTS Configuration
PIPER_PATH=piper\piper.exe
PIPER_MODEL=piper\piper-model.onnx
```

> **Tip:** You can use relative paths (as shown above) or absolute paths.

### 7. Folder structure

Your folder should look like this:

```
myLocalTTS/
├── local_tts_tool.exe
├── whisper-cli.exe
├── ggml-large-v3-turbo.bin
├── tts_config.txt                    (required for TTS - points to piper)
├── piper/                            (optional - for TTS)
│   ├── piper.exe
│   ├── espeak-ng.dll
│   ├── espeak-ng-data/
│   ├── onnxruntime.dll
│   ├── onnxruntime_providers_shared.dll
│   ├── piper_phonemize.dll
│   ├── libtashkeel_model.ort
│   ├── piper-model.onnx              (voice model)
│   └── piper-model.onnx.json         (voice model config)
├── *.dll (from whisper.cpp release)
```

## Usage

1. Run `local_tts_tool.exe`
2. Select your microphone from the list
3. Wait for the "Listening..." message

### Speech-to-Text (F9)

4. **Hold F9** to record your voice
5. **Release F9** to transcribe and auto-paste the text

The transcribed text will be automatically pasted into whatever application is currently focused.

### Text-to-Speech (F10)

4. **Select text** in any application (highlight it with your mouse or Shift+Arrow keys)
5. **Press F10** to have the selected text read aloud
6. **Press F10 again** while audio is playing to stop playback

## Building from Source

### Prerequisites

-   [Rust](https://rustup.rs/) (latest stable)
-   Windows 10/11

### Build

```bash
git clone https://github.com/marcelo-t-tomita/myLocalTTS.git
cd myLocalTTS
cargo build --release
```

The executable will be at `target/release/local_tts_tool.exe`.

## Configuration

### Text-to-Speech (Piper) Configuration

You can configure Piper paths using environment variables or a config file.

**Option 1: Environment Variables**

```powershell
$env:PIPER_PATH = "C:\path\to\piper.exe"
$env:PIPER_MODEL = "C:\path\to\model.onnx"
```

**Option 2: Config File**

Create a `tts_config.txt` file in the same folder as the executable:

```
# Piper TTS Configuration
PIPER_PATH=C:\path\to\piper.exe
PIPER_MODEL=C:\path\to\model.onnx
```

**Default Paths** (if not configured):
- `piper.exe` in the application folder
- `piper-model.onnx` in the application folder

### Changing the hotkey

Edit `src/main.rs`:
- F9 (Speech-to-Text): Line ~97
- F10 (Text-to-Speech): Line ~98

```rust
let is_f9_pressed = KeybdKey::F9Key.is_pressed();
let is_f10_pressed = KeybdKey::F10Key.is_pressed();
```

Available keys: `F1Key` through `F12Key`, or see [inputbot documentation](https://docs.rs/inputbot).

### Changing the Whisper model

Edit `src/main.rs` line 47:

```rust
let path_to_model = "ggml-large-v3-turbo.bin";
```

### Forcing a specific language

Edit `src/transcribe.rs` and add the language flag:

```rust
.arg("-l")
.arg("en") // or "pt", "es", "fr", etc.
```

## PowerShell Alias (Optional)

Add this to your PowerShell profile (`notepad $PROFILE`):

```powershell
function tts {
    Push-Location 'C:\path\to\myLocalTTS'
    .\local_tts_tool.exe
    Pop-Location
}
```

Then just type `tts` to start the tool.

## Troubleshooting

### "Model file not found"

Make sure the model file (e.g., `ggml-large-v3-turbo.bin`) is in the same folder as the executable.

### "Whisper executable not found"

Make sure `whisper-cli.exe` is in the same folder as the executable.

### "DLL not found" error

Copy all `.dll` files from the whisper.cpp release to the same folder as the executable.

### Transcription is slow

-   Use the CUDA version of whisper.cpp for GPU acceleration
-   Use a smaller model (e.g., `ggml-small.bin`)

### F9 key not working

-   Make sure no other application is using F9 as a global hotkey
-   Try running from a different terminal window

### "TTS narrator not available" warning

This is normal if you haven't configured Piper. Speech-to-Text (F9) will still work. To enable Text-to-Speech:

1. Download Piper from [releases](https://github.com/OHF-Voice/piper1-gpl/releases) - extract the **entire folder**, not just `piper.exe`
2. Download a voice model (`.onnx`) **and** its config file (`.onnx.json`) from [Piper Voices](https://huggingface.co/rhasspy/piper-voices)
3. Create a `tts_config.txt` file pointing to the paths (see Installation section)

### Piper fails with exit code -1073741515

This error means "DLL not found". Make sure you have all these files in your `piper` folder:
- `espeak-ng.dll`
- `espeak-ng-data/` folder (with all its contents)
- `onnxruntime.dll`
- `onnxruntime_providers_shared.dll`
- `piper_phonemize.dll`

### Piper fails with empty error message

Make sure:
- The `.onnx.json` config file exists alongside the `.onnx` model file
- Both files have the same base name (e.g., `piper-model.onnx` and `piper-model.onnx.json`)
- The `espeak-ng-data` folder is present

### F10 says "No text selected"

Make sure you have text selected (highlighted) in the active application before pressing F10. The tool simulates Ctrl+C to copy the selection.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

-   [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - High-performance C/C++ implementation of OpenAI's Whisper
-   [OpenAI Whisper](https://github.com/openai/whisper) - Original speech recognition model
-   [Piper](https://github.com/OHF-Voice/piper1-gpl) - Fast, local neural text-to-speech system
