use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(Debug, Deserialize, Serialize)]
struct Motd {
    msg: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct RespResult {
    motd: Motd,
    success: bool,
    result: f64,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
    #[command(description = "process a http test")]
    Test,
}

async fn get_exchange(from: &str, target: &str, value: &str) -> Result<RespResult, reqwest::Error> {
    let url = format!(
        "https://api.exchangerate.host/convert?from={from}&to={target}&amount={amount}",
        from = from,
        target = target,
        amount = value
    );
    let resp_result = reqwest::get(url).await;
    let resp = match resp_result {
        Ok(r) => r.text().await.unwrap(),
        Err(e) => return Err(e),
    };
    let res: RespResult = serde_json::from_str(&resp.as_str()).unwrap();
    return Ok(res);
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?
        }
        Command::Test => {
            let r = get_exchange("USD", "EUR", "2").await.unwrap();
            log::info!("{:?}", r);
            let text: &str = "處理的狀況";
            bot.send_message(msg.chat.id, text).await?
        }
    };

    Ok(())
}
