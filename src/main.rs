use async_std::task;
use ferrisgram::error::{GroupIteration, Result};
use ferrisgram::ext::filters::message;
use ferrisgram::ext::handlers::{CommandHandler, MessageHandler};
use ferrisgram::ext::{Context, Dispatcher, Updater};
use ferrisgram::types::{LinkPreviewOptions, MessageOrigin};
use ferrisgram::Bot;
use std::env;
use std::process::Command;
use std::time::Duration;

const TOKEN_ENV: &str = "TOKEN";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var(TOKEN_ENV).expect("Environment variable TOKEN is not set");

    let bot = match Bot::new(&token, None).await {
        Ok(bot) => bot,
        Err(error) => panic!("failed to create bot: {}", error),
    };

    let dispatcher = &mut Dispatcher::new(&bot);
    dispatcher.add_handler(CommandHandler::new("start", start));
    dispatcher.add_handler(CommandHandler::new("ping", pingh));
    dispatcher.add_handler(CommandHandler::new("id", getid));
    dispatcher.add_handler(CommandHandler::new("sleep", sysnchk));
    dispatcher.add_handler_to_group(MessageHandler::new(echo, message::All::filter()), 1);

    let mut updater = Updater::new(&bot, dispatcher);
    updater.allowed_updates = Some(vec!["message"]);

    tokio::spawn(async move {
        task::sleep(Duration::from_secs(21600)).await;

        let self_executable = match env::current_exe() {
            Ok(path) => path,
            Err(err) => {
                eprintln!("Failed to get current executable: {}", err);
                return;
            }
        };

        let args: Vec<String> = env::args().collect();
        let status = Command::new(self_executable)
            .args(&args[1..])
            .envs(env::vars())
            .status();

        match status {
            Ok(status) => eprintln!("Process exited with non-zero status: {:?}", status),
            Err(err) => eprintln!("Failed to exec the process: {}", err),
        }
    });

    println!("Started!");
    updater.start_polling(false).await.ok();
    println!("Bye!");
}

async fn start(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let mut link_preview_options = LinkPreviewOptions::new();
    link_preview_options.is_disabled = Some(true);

    msg.reply(
        &bot,
        "Hey! I am an echo bot built in love with [Ferrisgram](https://github.com/ferrisgram/ferrisgram). I will just repeat your messages.",
    )
    .parse_mode("markdown".to_string())
    .link_preview_options(link_preview_options)
    .send()
    .await?;

    Ok(GroupIteration::EndGroups)
}

async fn echo(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let chat = ctx.effective_chat.unwrap();
    let msg = ctx.effective_message.unwrap();
    bot.copy_message(chat.id, chat.id, msg.message_id)
        .send()
        .await?;
    Ok(GroupIteration::EndGroups)
}

async fn sysnchk(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let emsg = msg.reply(&bot, "Sleeping 10s ...").send().await?;
    task::sleep(Duration::from_secs(10)).await;

    bot.edit_message_text("Done!".to_string())
        .chat_id(msg.chat.id)
        .message_id(emsg.message_id)
        .send()
        .await?;

    Ok(GroupIteration::EndGroups)
}

async fn pingh(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let start_time = std::time::Instant::now();
    let emsg = msg.reply(&bot, "Pinging ...").send().await?;
    let elapsed_time = start_time.elapsed();

    bot.edit_message_text(format!("Pong!\n{:?}!", elapsed_time))
        .chat_id(msg.chat.id)
        .message_id(emsg.message_id)
        .send()
        .await?;

    Ok(GroupIteration::EndGroups)
}

async fn getid(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let argsctx = ctx.clone();
    let msg = ctx.effective_message.unwrap();
    let user = ctx.effective_user.unwrap();

    let mut sendtxt = format!(
        "<b>Chat ID:</b> <code>{}</code>\n<b>Message ID:</b> <code>{}</code>\n<b>My ID:</b> <code>{}</code>\n<b>Your ID:</b> <code>{}</code>\n",
        msg.chat.id,
        msg.message_id,
        bot.user.id,
        user.id
    );

    if let Some(rmsg) = msg.reply_to_message.clone() {
        sendtxt.push_str(&format!(
            "<b>Replied Message ID:</b> <code>{}</code>\n",
            rmsg.message_id
        ));

        if let Some(rusr) = rmsg.from.clone() {
            sendtxt.push_str(&format!(
                "<b>Replied User ID:</b> <code>{}</code>\n",
                rusr.id
            ));
        }

        if let Some(rusc) = rmsg.sender_chat.clone() {
            sendtxt.push_str(&format!(
                "<b>Replied Chat ID:</b> <code>{}</code>\n",
                rusc.id
            ));
        }

        if let Some(forward_origin) = rmsg.forward_origin.clone() {
            match forward_origin {
                MessageOrigin::MessageOriginUser(user_origin) => {
                    sendtxt.push_str(&format!(
                        "<b>Forwarded From User ID:</b> <code>{}</code>\n",
                        user_origin.sender_user.id,
                    ));
                }
                MessageOrigin::MessageOriginChat(chat_origin) => {
                    sendtxt.push_str(&format!(
                        "<b>Forwarded From Chat ID:</b> <code>{}</code>\n",
                        chat_origin.sender_chat.id,
                    ));
                }
                MessageOrigin::MessageOriginChannel(channel_origin) => {
                    sendtxt.push_str(&format!(
                        "<b>Forwarded Message ID:</b> <code>{}</code>\n<b>Forwarded From Channel ID:</b> <code>{}</code>\n",
                        channel_origin.message_id,
                        channel_origin.chat.id,
                    ));
                }
                _ => {}
            }
        }
    }

    let args = &argsctx.args()[1..];
    if !args.is_empty() {
        if let Ok(chatid) = args[0].parse::<i64>() {
            if chatid != 0 {
                if let Ok(chat) = bot.get_chat(chatid).send().await {
                    sendtxt.push_str(&format!(
                        "<b>Chat Name:</b> <code>{}</code>\n<b>Chat Username:</b> @{}\n",
                        chat.title
                            .unwrap_or(chat.first_name.unwrap_or("None".to_string())),
                        chat.username.unwrap_or("None".to_string()),
                    ));
                }
            }
        }
    }

    msg.reply(&bot, &sendtxt)
        .parse_mode("html".to_string())
        .send()
        .await?;

    Ok(GroupIteration::EndGroups)
}
