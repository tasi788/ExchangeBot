use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
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
    let res = reqwest::get(url).await?.json::<Symbols>().await?;
    Ok(res)
}

async fn get_exchange(from: &str, target: &str, value: &str) -> Result<RespResult, reqwest::Error> {
    let url = format!(
        "https://api.exchangerate.host/convert?from={from}&to={target}&amount={amount}",
        from = from,
        target = target,
        amount = value
    );
    let res = reqwest::get(url).await?.json::<RespResult>().await?;
    Ok(res)
}

async fn answer(bot: Bot, msg: Message, cmd: Command, support: Symbols) -> ResponseResult<()> {
    let text = match cmd {
        Command::Help => Command::descriptions().to_string(),
        Command::Username(username) => format!("Your username is @{username}."),
        Command::UsernameAndAge { username, age } => {
            format!("Your username is @{username} and age is {age}.")
        }
        Command::Ex { query } => {
            // ensure the regex is compiled exactly once
            static RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new(
                    r"(?x)                       # Free-spacing mode
                      (?P<amount>\d+|\d+\.\d+|)  # Amount
                      (?P<from>\S{1,4})          # From
                      =                          # Sep
                      (?P<target>\S{1,4})        # Target
                    ",
                )
                .unwrap()
            });

            let text = match RE.captures(&query) {
                Some(caps) if !support.symbols.contains_key(&caps["from"].to_uppercase()) => {
                    format!("不支援的幣別 `{from}`", from = &caps["from"])
                }
                Some(caps) if !support.symbols.contains_key(&caps["target"].to_uppercase()) => {
                    format!("不支援的幣別 `{target}`", target = &caps["target"])
                }
                Some(caps) => {
                    let amount = &caps["amount"];
                    let from = &caps["from"];
                    let target = &caps["target"];
                    get_exchange(from, target, amount).await.map(|r| {
                        format!(
                            "`{source}` `{from}` 對 `{target}` 的匯率為 `{amount:.2}` ",
                            source = amount.to_uppercase(),
                            from = from.to_uppercase(),
                            target = target.to_uppercase(),
                            amount = r.result
                        )
                    })?
                }
                None => "Invalid format, Expected `{Amount?}{From}={Target}`".to_string(),
            };

            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_to_message_id(msg.id)
                .await?;
            return Ok(());
        }
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
