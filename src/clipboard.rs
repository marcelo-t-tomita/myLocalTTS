use anyhow::Result;
use arboard::Clipboard;
use enigo::{Enigo, Key, Settings, Direction, Keyboard}; 
use std::thread;
use std::time::Duration;

pub struct ClipboardManager {
    clipboard: Clipboard,
    enigo: Enigo,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new().map_err(|e| anyhow::anyhow!("Failed to init clipboard: {}", e))?;
        // Enigo 0.2.x constructor takes Settings
        let enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!("Failed to init enigo: {:?}", e))?;
        Ok(Self { clipboard, enigo })
    }

    pub fn paste_text(&mut self, text: &str) -> Result<()> {
        // 1. Set text to clipboard
        self.clipboard.set_text(text.to_owned()).map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;
        
        // 2. Simulate CTRL+V
        thread::sleep(Duration::from_millis(100));
        
        // Press Control
        self.enigo.key(Key::Control, Direction::Press).map_err(|e| anyhow::anyhow!("Enigo error: {:?}", e))?;
        // Click V (Unicode) - Note: Key::Layout('v') in older vers, 0.2 uses different variants usually.
        // Assuming Key::Unicode('v') or Key::V exists. In modern Enigo 0.2, standard keys are often enum variants like Key::V.
        // Let's try Key::V first, if not available we'll try Unicode.
        // Actually, for 'v' it's often Key::V or use text("v") but text sends strings.
        // Let's try Key::Unicode('v') as it is common for chars.
        // UPDATE: Error log said Key::Layout not found.
        // Common 0.2 API: Key::V might exist.
        // Let's use `text("v")`? No, that types V. We need to hold CTRL.
        // Let's check common keys. Key::Control exists. Key::V might not.
        // But Key::Unicode('v') usually exists.
        
        // However, to be safe against API changes, let's use what we know exists or try Key::Unicode.
        self.enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| anyhow::anyhow!("Enigo error: {:?}", e))?;
        
        // Release Control
        self.enigo.key(Key::Control, Direction::Release).map_err(|e| anyhow::anyhow!("Enigo error: {:?}", e))?;

        Ok(())
    }
}
