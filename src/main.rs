use std::env;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use async_std::sync::Mutex;
use async_std::task;
use ferrisgram::error::{GroupIteration, Result};
use ferrisgram::ext::filters::{chat_join_request, message};
use ferrisgram::ext::handlers::{ChatJoinRequestHandler, CommandHandler, MessageHandler};
use ferrisgram::ext::{Context, Dispatcher, Updater};
use ferrisgram::types::{ChatFullInfo, LinkPreviewOptions, MessageOrigin, Update};
use ferrisgram::Bot;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming as Body};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use regex::Regex;
use tokio::net::TcpListener;

const TOKEN_ENV: &str = "TOKEN";
const SECRET: &str = "sexm";
const PORT: &str = "PORT";
const WEB_URL: &str = "URL";

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"(http(s)?://)?(t|telegram)\.(me|dog)/").unwrap();
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = Arc::new(env::var(TOKEN_ENV).expect("TOKEN env not set!"));
    let real_bot = Bot::new(&token, None).await.expect("failed to init bot");
    let bot: &'static Bot = Box::leak(Box::new(real_bot));
    tokio::spawn(async {
        task::sleep(Duration::from_secs(21600)).await;
        let exe = env::current_exe().unwrap();
        let args: Vec<_> = env::args().skip(1).collect();
        let _ = Command::new(exe).args(&args).envs(env::vars()).status();
    });
    let weburl = env::var(WEB_URL).unwrap_or_default();
    if !weburl.is_empty() {
        let port = env::var(PORT)
            .unwrap_or_else(|_| "8080".into())
            .parse::<u16>()
            .expect("PORT must be a number");
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await.expect("bind failed!");
        let ok = bot
            .set_webhook::<String>(weburl + &token)
            .secret_token(SECRET.into())
            .drop_pending_updates(false)
            .max_connections(40)
            .allowed_updates(vec!["message".into(), "chat_join_request".into()])
            .send()
            .await
            .unwrap();
        println!("Webhook set: {ok}\nStarted!");
        let webhook_dispatcher = Arc::new(Mutex::new(setup_dispatcher(bot)));
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let dispatcher = webhook_dispatcher.clone();
            let token = token.clone();
            let io = TokioIo::new(stream);
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_webhook(req, dispatcher.clone(), token.clone())),
                )
                .await
            {
                eprintln!("serve_connection error: {err:?}");
            }
        }
    } else {
        let raw_dispatcher = &mut setup_dispatcher(bot);
        let mut updater = Updater::new(&bot, raw_dispatcher);
        updater.allowed_updates = Some(vec!["message", "chat_join_request"]);
        println!("Started!");
        updater.start_polling(false).await.ok();
    }
    println!("Bye!");
}

fn setup_dispatcher(bot: &'static Bot) -> Dispatcher<'static> {
    let mut dispatcher = Dispatcher::new(bot);
    dispatcher.add_handler(CommandHandler::new("start", start));
    dispatcher.add_handler(CommandHandler::new("ping", pingh));
    dispatcher.add_handler(CommandHandler::new("id", getid));
    dispatcher.add_handler(CommandHandler::new("sleep", sysnchk));
    dispatcher.add_handler(ChatJoinRequestHandler::new(
        autoapprove,
        chat_join_request::All::filter(),
    ));
    dispatcher.add_handler_to_group(MessageHandler::new(echo, message::All::filter()), 1);
    dispatcher
}

async fn handle_webhook(
    req: Request<Body>,
    dispatcher: Arc<Mutex<Dispatcher<'static>>>,
    token: Arc<String>,
) -> std::result::Result<Response<Full<Bytes>>, hyper::Error> {
    if req.method() != Method::POST {
        return Ok(resp(StatusCode::METHOD_NOT_ALLOWED, b"Method Not Allowed!"));
    }
    let path = req.uri().path();
    if path.strip_prefix("/").unwrap_or("") != &**token {
        return Ok(resp(StatusCode::UNAUTHORIZED, b"Invalid path!"));
    }
    if req
        .headers()
        .get("X-Telegram-Bot-Api-Secret-Token")
        .map(|v| v != SECRET)
        .unwrap_or(true)
    {
        return Ok(resp(StatusCode::UNAUTHORIZED, b"Unauthorized!"));
    }
    let body = req.collect().await?.to_bytes();
    let update: Update = match serde_json::from_slice(&body) {
        Ok(upd) => upd,
        Err(_) => return Ok(resp(StatusCode::BAD_REQUEST, b"Bad JSON!")),
    };
    tokio::spawn({
        let dispatcher = dispatcher.clone();
        async move {
            let mut dispatcher = dispatcher.lock().await;
            let handle = dispatcher.process_update(&update);
            let _ = handle.await;
        }
    });
    Ok(resp(StatusCode::OK, b"OK"))
}

fn resp(status: StatusCode, msg: &'static [u8]) -> Response<Full<Bytes>> {
    println!("{}", String::from_utf8_lossy(msg));
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from_static(msg)))
        .unwrap()
}

async fn start(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let msg = ctx.effective_message.unwrap();
    let mut link_preview_options = LinkPreviewOptions::new();
    link_preview_options.is_disabled = Some(true);
    msg.reply(
        &bot,
        "Hey! I am an echo bot built in love with [Ferrisgram](https://github.com/ferrisgram/ferrisgram). I will just repeat your messages, approve chant join requests and give you ids through /id, give pings through /ping :)",
    )
    .parse_mode("markdown".to_string())
    .link_preview_options(link_preview_options)
    .send()
    .await?;
    Ok(GroupIteration::EndGroups)
}

async fn echo(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let user = ctx.effective_user.unwrap();
    let chat = ctx.effective_chat.unwrap();
    if user.id == 5844597230 {
        bot.ban_chat_member(chat.id, user.id).send().await?;
        return Ok(GroupIteration::EndGroups);
    }
    let msg = ctx.effective_message.unwrap();
    bot.copy_message(chat.id, chat.id, msg.message_id)
        .send()
        .await?;
    Ok(GroupIteration::EndGroups)
}

async fn sysnchk(bot: Bot, ctx: Context) -> Result<GroupIteration> {
    let user = ctx.effective_user.unwrap();
    if user.id != 1594433798 {
        return Ok(GroupIteration::EndGroups);
    }
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

pub async fn getchat(bot: &Bot, arg: &str) -> (Option<ChatFullInfo>, String) {
    let mut chat_id = arg.to_string();
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
        let (chat, _) = getchat(&bot, &arg).await;
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
