use ferrisgram::error::{GroupIteration, Result};
use ferrisgram::ext::filters::message;
use ferrisgram::ext::handlers::{CommandHandler, MessageHandler};
use ferrisgram::ext::{Context, Dispatcher, Updater};
use ferrisgram::types::LinkPreviewOptions;
use ferrisgram::Bot;

#[allow(unused)]
#[tokio::main]
async fn main() {
    let token = "1883187841:AAHfaaBAWVAYyORj0-JLR3Q6HnM_8_xz7PI";
    let bot = match Bot::new(&token, None).await {
        Ok(bot) => bot,
        Err(error) => panic!("failed to create bot: {}", error),
    };
    let mut dispatcher = &mut Dispatcher::new(&bot);
    dispatcher.add_handler(CommandHandler::new("start", start));
    dispatcher.add_handler_to_group(
        MessageHandler::new(echo, message::Text::filter().or(message::Caption::filter())),
        1,
    );
    let mut updater = Updater::new(&bot, dispatcher);
    updater.start_polling(false).await;
}

async fn start(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let mut link_preview_options = LinkPreviewOptions::new();
    link_preview_options.is_disabled = Some(true);
    msg.reply(
        &bot,
        "Hey! I am an echo bot built using [Ferrisgram](https://github.com/ferrisgram/ferrisgram).
I will repeat your messages.",
    )
    .parse_mode("markdown".to_string())
    .link_preview_options(link_preview_options)
    // You must use this send() method in order to send the request to the API
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
