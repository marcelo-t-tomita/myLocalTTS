# LocalTTS

A lightweight, offline speech-to-text tool for Windows that runs locally using [Whisper.cpp](https://github.com/ggerganov/whisper.cpp). Press a hotkey to record your voice, release to transcribe, and the text is automatically pasted into your active application.

## Features

- **Fully Offline** - No internet connection required, all processing happens locally
- **GPU Accelerated** - Supports CUDA for fast transcription on NVIDIA GPUs
- **Auto Language Detection** - Automatically detects the spoken language
- **Simple Hotkey** - Hold F9 to record, release to transcribe and paste
- **Microphone Selection** - Choose your preferred input device at startup
- **Low Latency** - Text appears almost instantly after releasing the hotkey

## Requirements

- Windows 10/11
- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp/releases) executable (`whisper-cli.exe`)
- A Whisper model file (e.g., `ggml-large-v3-turbo.bin`)
- For GPU acceleration: NVIDIA GPU with CUDA support

## Installation

### 1. Download the release

Download the latest release from the [Releases](https://github.com/yourusername/localTTS/releases) page.

### 2. Download Whisper.cpp

> **Note:** The Whisper executable and DLLs are **not included** in this repository due to their size and licensing. You must download them separately.

1. Go to [whisper.cpp releases](https://github.com/ggerganov/whisper.cpp/releases)
2. Download the appropriate version:
   - **CPU only**: `whisper-bin-x64.zip`
   - **NVIDIA GPU (recommended)**: `whisper-cublas-12.4.0-bin-x64.zip`

   > **Tip:** The `whisper-cublas-12.4.0-bin-x64.zip` version has been tested and works best for CUDA GPU acceleration.
3. Extract the ZIP file
4. Copy the following files to your LocalTTS folder:
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

| Model | Size | Quality | Speed |
|-------|------|---------|-------|
| `ggml-tiny.bin` | ~75 MB | Basic | Fastest |
| `ggml-base.bin` | ~147 MB | Good | Fast |
| `ggml-small.bin` | ~466 MB | Better | Medium |
| `ggml-medium.bin` | ~1.5 GB | Great | Slower |
| `ggml-large-v3-turbo.bin` | ~1.6 GB | Best | Fast (GPU) |

For best results with a GPU, use `ggml-large-v3-turbo.bin`:

```powershell
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin" -OutFile "ggml-large-v3-turbo.bin"
```

### 4. Folder structure

Your folder should look like this:

```
localTTS/
├── local_tts_tool.exe
├── whisper-cli.exe
├── ggml-large-v3-turbo.bin
├── *.dll (from whisper.cpp release)
```

## Usage

1. Run `local_tts_tool.exe`
2. Select your microphone from the list
3. Wait for "Listening for F9" message
4. **Hold F9** to record your voice
5. **Release F9** to transcribe and auto-paste the text

The transcribed text will be automatically pasted into whatever application is currently focused.

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- Windows 10/11

### Build

```bash
git clone https://github.com/yourusername/localTTS.git
cd localTTS
cargo build --release
```

The executable will be at `target/release/local_tts_tool.exe`.

## Configuration

### Changing the hotkey

Edit `src/main.rs` line 75:

```rust
let is_pressed = KeybdKey::F9Key.is_pressed();
```

Available keys: `F1Key` through `F12Key`, or see [inputbot documentation](https://docs.rs/inputbot).

### Changing the model

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
    Push-Location 'C:\path\to\localTTS'
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
- Use the CUDA version of whisper.cpp for GPU acceleration
- Use a smaller model (e.g., `ggml-small.bin`)

### F9 key not working
- Make sure no other application is using F9 as a global hotkey
- Try running from a different terminal window

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - High-performance C/C++ implementation of OpenAI's Whisper
- [OpenAI Whisper](https://github.com/openai/whisper) - Original speech recognition model
