use async_std::task;
use ferrisgram::error::{GroupIteration, Result};
use ferrisgram::ext::filters::message;
use ferrisgram::ext::handlers::{CommandHandler, MessageHandler};
use ferrisgram::ext::{Context, Dispatcher, Updater};
use ferrisgram::types::LinkPreviewOptions;
use ferrisgram::Bot;
use std::env;
use std::time::Duration;

#[allow(unused)]
#[tokio::main]
async fn main() {
    let token = env::var("TOKEN")
        .unwrap_or_else(|_| "YOU MAY PUT THE TOKEN HERE - For hardcode lovers.".to_string());
    let bot = match Bot::new(&token, None).await {
        Ok(bot) => bot,
        Err(error) => panic!("failed to create bot: {}", error),
    };
    let mut dispatcher = &mut Dispatcher::new(&bot);
    dispatcher.add_handler(CommandHandler::new("start", start));
    dispatcher.add_handler(CommandHandler::new("ping", pingh));
    dispatcher.add_handler(CommandHandler::new("sleep", sysnchk));
    dispatcher.add_handler_to_group(
        MessageHandler::new(echo, message::Text::filter().or(message::Caption::filter())),
        1,
    );
    let mut updater = Updater::new(&bot, dispatcher);
    updater.allowed_updates = Some(vec!["message"]);
    println!("Started!");
    updater.start_polling(false).await;
    println!("Bye!");
}

async fn start(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let mut link_preview_options = LinkPreviewOptions::new();
    link_preview_options.is_disabled = Some(true);
    msg.reply(
        &bot,
        "Hey! I am an echo bot built in love with [Ferrisgram](https://github.com/ferrisgram/ferrisgram).
I will just repeat your messages.",
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

async fn sysnchk(b: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let emsg = msg.reply(&b, "Sleeping 10s ...").send().await?;
    task::sleep(Duration::from_secs(10)).await;
    b.edit_message_text("Done!".to_string())
        .chat_id(msg.chat.id)
        .message_id(emsg.message_id)
        .send()
        .await?;
    Ok(GroupIteration::EndGroups)
}

async fn pingh(b: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let startt = std::time::Instant::now();
    let emsg = msg.reply(&b, "Pinging ...").send().await?;
    let endt = std::time::Instant::now();
    let elapsed_time = endt - startt;
    b.edit_message_text(format!("Pong!\n{:?}!", elapsed_time))
        .chat_id(msg.chat.id)
        .message_id(emsg.message_id)
        .send()
        .await?;
    Ok(GroupIteration::EndGroups)
}
