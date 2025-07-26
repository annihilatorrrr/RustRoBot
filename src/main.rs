use std::env;
use std::process::Command;
use std::time::Duration;

use async_std::task;
use ferrisgram::error::{GroupIteration, Result};
use ferrisgram::ext::filters::chat_join_request;
use ferrisgram::ext::filters::message;
use ferrisgram::ext::handlers::{ChatJoinRequestHandler, CommandHandler, MessageHandler};
use ferrisgram::ext::{Context, Dispatcher, Updater};
use ferrisgram::types::{ChatFullInfo, LinkPreviewOptions, MessageOrigin};
use ferrisgram::Bot;
use regex::Regex;

const TOKEN_ENV: &str = "TOKEN";
lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"(http(s)?://)?(t|telegram)\.(me|dog)/").unwrap();
}

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
    dispatcher.add_handler(ChatJoinRequestHandler::new(
        autoapprove,
        chat_join_request::All::filter(),
    ));
    dispatcher.add_handler_to_group(MessageHandler::new(echo, message::All::filter()), 1);

    let mut updater = Updater::new(&bot, dispatcher);
    updater.allowed_updates = Some(vec!["message", "chat_join_request"]);

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

async fn autoapprove(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let joinrequest = ctx.update.chat_join_request.unwrap();
    bot.copy_message(joinrequest.chat.id, -1001552278027, 6510)
        .send()
        .await?;
    bot.approve_chat_join_request(joinrequest.chat.id, joinrequest.from.id)
        .send()
        .await?;
    Ok(GroupIteration::EndGroups)
}

pub async fn getchat(bot: &Bot, arg: String) -> (Option<ChatFullInfo>, String) {
    let mut chat_id = arg.clone();
    if chat_id.parse::<i64>().is_err() {
        chat_id = USERNAME_REGEX.replace(&chat_id, "@").into_owned();
        if !chat_id.starts_with('@') {
            chat_id.insert(0, '@');
        }
    }
    let payload = serde_json::json!({ "chat_id": chat_id });
    let res = bot.get("getChat", Some(&payload)).await;
    match res {
        Ok(chat) => match serde_json::from_value::<ChatFullInfo>(chat) {
            Ok(parsed_chat) => (Some(parsed_chat), "".to_string()),
            Err(_) => (None, "BadRequest chat not found!".to_string()),
        },
        Err(_) => (None, "BadRequest unable to make the Request!".to_string()),
    }
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
        let arg = args[0].to_string();
        let (chat, _) = getchat(&bot, arg).await;
        if let Some(chat) = chat {
            sendtxt.push_str(&format!(
                "<b>{}'s ID:</b> <code>{}</code>\n<b>{}'s Title:</b> <code>{}</code>\n",
                arg,
                chat.id,
                arg,
                chat.title
                    .unwrap_or(chat.first_name.unwrap_or("None".to_string())),
            ));
            if let Some(username) = chat.username {
                sendtxt.push_str(&format!("<b>{}'s Username:</b> @{}\n", arg, username));
            }
        } else {
            sendtxt.push_str("<b>Error:</b> Unable to GetChat!");
        }
    }

    msg.reply(&bot, &sendtxt)
        .parse_mode("html".to_string())
        .send()
        .await?;

    Ok(GroupIteration::EndGroups)
}
