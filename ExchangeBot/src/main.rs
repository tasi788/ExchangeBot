use regex;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

#[derive(Debug, Deserialize)]
struct RespResult {
    result: f64,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let support: Symbols = get_support().await.unwrap();
    let bot = Bot::from_env();

    Command::repl(bot, move |bot, msg, cmd| {
        answer(bot, msg, cmd, support.clone())
    })
    .await;
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
    #[command(description = "exchange")]
    Ex { query: String },
}

#[derive(Debug, Deserialize, Clone)]
struct CurrencyInfo {
    description: String,
    code: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Symbols {
    symbols: std::collections::HashMap<String, CurrencyInfo>,
}

async fn get_support() -> Result<Symbols, reqwest::Error> {
    let url = "https://api.exchangerate.host/symbols";

    let resp_result = reqwest::get(url).await?.text().await?;
    let res: Symbols = serde_json::from_str(&resp_result.as_str()).unwrap();
    return Ok(res);
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

async fn answer(bot: Bot, msg: Message, cmd: Command, support: Symbols) -> ResponseResult<()> {
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
        Command::Ex { query } => {
            let symbols = support.symbols.clone();
            let re = regex::Regex::new(r"(\d+|\d+\.\d+|)(\S{1,4})=(\S{1,4})").unwrap();
            let upper_query = query.to_uppercase();
            let caps = re.captures(&upper_query).unwrap();
            let amount = caps.get(1).unwrap().as_str();
            let from = caps.get(2).unwrap().as_str();
            let target = caps.get(3).unwrap().as_str();

            match symbols.get(from) {
                Some(v) => {
                    let r = get_exchange(from, target, amount).await.unwrap();
                    let text = format!(
                        "`{source}` `{from}` 對 `{target}` 的匯率為 `{amount:.2}` ",
                        source = amount.to_uppercase(),
                        from = from.to_uppercase(),
                        target = target.to_uppercase(),
                        amount = r.result);
                    bot.send_message(msg.chat.id, &text)
                        .parse_mode(ParseMode::MarkdownV2)
                        .reply_to_message_id(msg.id)
                        .await?
                },
                None => { bot.send_message(msg.chat.id, "不支援的幣別")
                        .parse_mode(ParseMode::MarkdownV2)
                        .reply_to_message_id(msg.id)
                        .await? },
            }
        }
    };

    Ok(())
}
