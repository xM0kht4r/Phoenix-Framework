use anyhow::{Result, Context};

pub async fn cmd(cmd: &str) -> Result<String> {

    let output = std::process::Command::new("cmd")
            .args(&["/C", cmd])
            .output()?;
    
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } 
    else {
        return Ok(String::from_utf8_lossy(&output.stderr).to_string())
    }

}