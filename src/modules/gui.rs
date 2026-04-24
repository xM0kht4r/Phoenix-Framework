use anyhow::{Result, bail, Context};
use std::ptr;
use std::ffi::CString;
use winapi::um::winuser::{MessageBoxA, MB_OK};

pub async fn msg(msg: &str) -> Result<()>{
	
	let title = CString::new("Message!").unwrap();
	let msg = CString::new(msg).context("Failed to convet message to CString")?;
	unsafe {MessageBoxA(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK)};

	Ok(())
}