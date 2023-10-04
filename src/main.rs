use exchange::Exchange;
use grammers_client;
use grammers_client::{Client, Config, InputMessage, SignInError, Update};
use grammers_session::Session;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{self, BufRead as _, Write as _};
use tokio::{runtime, task};

use exchange_bot as lib;
mod exchange;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;
type Results<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SESSION_FILE: &str = "exchangebot.session";


fn prompt(message: &str) -> Results<String> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line)
}

fn command_filter(text: &str) -> Results<Option<(&str, &str)>> {
    match text.split_once(|c| char::is_whitespace(c)) {
        Some((cmd @ "/ec", args)) => Ok(Some((cmd, args))),
        Some(_) | None => Ok(None),
    }
}

fn parse_exchange_args(text: &str) -> Option<(String, String, String)> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?x)                      # Free-spacing mode
            ^\s*
            (?P<amount>\d+|\d+\.\d+|)   # Amount
            (?:\s*)                     # Optional Sep
            (?P<from>[a-zA-Z]{1,4})     # From
            (?:\s*=\s*|\s+)             # Sep
            (?:\s*)                     # Optional Sep
            (?P<target>[a-zA-Z]\S{1,4}) # Target
            \s*$
            ",
        )
        .unwrap()
    });
    match RE.captures(text.trim()) {
        Some(caps) => Some((
            caps["amount"].into(),
            caps["from"].into(),
            caps["target"].into(),
        )),
        None => None,
    }
}

async fn handle_ex_command(cmd_args: &str, support: &exchange::Symbols, exchange_client: exchange::ExchangeClient) -> Results<String> {
    let text = match parse_exchange_args(cmd_args) {
        Some((_, from, _)) if !support.symbols.contains_key(&from.to_uppercase()) => {
            format!("不支援的幣別 `{from}`")
        }
        Some((_, _, target)) if !support.symbols.contains_key(&target.to_uppercase()) => {
            format!("不支援的幣別 `{target}`")
        }
        Some((amount, from, target)) => exchange_client.convert(&from, &target, &amount)
            .await
            .map(|r| {
                format!(
                    "`{amount}` `{from}` 對 `{target}` 的匯率為 `{result:.2}` ",
                    amount = amount,
                    from = from.to_uppercase(),
                    target = target.to_uppercase(),
                    result = r.result
                )
            })
            .unwrap(),
        None => {
            "不合法的格式, 應為 `{Amount?}{From}={Target}` 或 `{Amount?}{From} {Target}`".into()
        }
    };
    Ok(text)
}

async fn handle_update(_client: Client, update: Update, support: exchange::Symbols, exchange_client: exchange::ExchangeClient) -> Result {
    match update {
        Update::NewMessage(message) => {
            let filtered = command_filter(message.text())?;
            match filtered {
                Some(("/ec", cmd_args)) => {
                    let text = handle_ex_command(cmd_args, &support, exchange_client).await?;
                    let sent = message
                        .reply(InputMessage::text("查詢中..."))
                        .await
                        .unwrap();
                    sent.edit(InputMessage::markdown(text)).await.unwrap();
                }
                Some(_) | None => {}
            }
        }
        _ => {}
    }
    Ok(())
}

async fn async_main() -> Result {
    let config: lib::Config = lib::utils::load_config().unwrap();
    println!("Connecting to Telegram...");
    let client = Client::connect(Config {
        session: Session::load_file_or_create(SESSION_FILE)?,
        api_id: config.api_id,
        api_hash: config.clone().api_hash,
        params: Default::default(),
    })
    .await?;
    if !client.is_authorized().await? {
        println!("Signing in...");
        let phone = prompt("Enter your phone number (international format): ")?;
        let token = client
            .request_login_code(&phone, config.api_id, &config.clone().api_hash)
            .await?;
        let code = prompt("Enter the code you received: ")?;
        let signed_in = client.sign_in(&token, &code).await;
        match signed_in {
            Err(SignInError::PasswordRequired(password_token)) => {
                // Note: this `prompt` method will echo the password in the console.
                //       Real code might want to use a better way to handle this.
                let hint = password_token.hint().unwrap_or("None");
                let prompt_message = format!("Enter the password (hint {}): ", &hint);
                let password = prompt(prompt_message.as_str())?;

                client
                    .check_password(password_token, password.trim())
                    .await?;
            }
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
        println!("Signed in!");
    }
    println!("Connected!");

    println!("Waiting for messages...");
    while let Some(update) = tokio::select! {
        _ = tokio::signal::ctrl_c() => Ok(None),
        result = client.next_update() => result,
    }? {
        let handle = client.clone();
        let exchange_client = exchange::ExchangeClient::new(&config.api_token);
        let symbol = exchange_client.clone().get_list().await.unwrap();
        task::spawn(async move {
            match handle_update(handle, update, symbol, exchange_client).await {
                Ok(_) => {}
                Err(e) => eprintln!("Error handling updates!: {}", e),
            }
        });
    }
    println!("Saving session file and exiting...");
    client.session().save_to_file(SESSION_FILE)?;
    Ok(())
}

fn main() -> Result {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]  
    async fn client_get_list() {
        let config = lib::utils::load_config().unwrap();
        
        let client = exchange::ExchangeClient::new(&config.api_token);
        let result = client.get_list().await.unwrap();
        assert_ne!(result.symbols.len(), 0);
    }

    #[test]
    fn command_filter_ex_command() {
        assert_eq!(command_filter("/ec ").unwrap(), Some(("/ec", "")));
        assert_eq!(command_filter("/ec test").unwrap(), Some(("/ec", "test")));
        assert_eq!(
            command_filter("/ec test other args").unwrap(),
            Some(("/ec", "test other args"))
        );
    }

    #[test]
    fn command_filter_ignore() {
        assert_eq!(command_filter("/ec").unwrap(), None);
        assert_eq!(command_filter("/ectest").unwrap(), None);
        assert_eq!(command_filter("/start").unwrap(), None);
        assert_eq!(command_filter("/help").unwrap(), None);
    }

    #[test]
    fn parse_valid_ex_args_with_equal() {
        let stringify = |a: &str, b: &str, c: &str| (a.into(), b.into(), c.into());
        assert_eq!(
            parse_exchange_args("USD=TWD").unwrap(),
            stringify("", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("99USD=TWD").unwrap(),
            stringify("99", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("55.66USD=TWD").unwrap(),
            stringify("55.66", "USD", "TWD")
        );
    }

    #[test]
    fn parse_valid_ex_args_with_whitespace() {
        let stringify = |a: &str, b: &str, c: &str| (a.into(), b.into(), c.into());
        assert_eq!(
            parse_exchange_args("USD TWD").unwrap(),
            stringify("", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("99 USD=TWD").unwrap(),
            stringify("99", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("99 USD TWD").unwrap(),
            stringify("99", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("55.66 USD TWD").unwrap(),
            stringify("55.66", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("  55.66   USD   TWD   ").unwrap(),
            stringify("55.66", "USD", "TWD")
        );
        assert_eq!(
            parse_exchange_args("\n   55.66 \t  USD  \t\n  TWD  \n ").unwrap(),
            stringify("55.66", "USD", "TWD")
        );
    }

    #[test]
    fn parse_invalid_ex_args() {
        assert_eq!(parse_exchange_args("TWD"), None);
        assert_eq!(parse_exchange_args("=TWD"), None);
        assert_eq!(parse_exchange_args("99TWD"), None);
        assert_eq!(parse_exchange_args("99TWD="), None);
        assert_eq!(parse_exchange_args("99FUTAFUTA=TWD"), None);
        assert_eq!(parse_exchange_args("99TWD=FUTAFUTA"), None);
    }

    // #[tokio::test]
    // async fn ex_missing_from() {
    //     let support = exchange::Symbols {
    //         symbols: HashMap::from([(
    //             "TWD".into(),
    //             exchange::CurrencyInfo {
    //                 description: "".into(),
    //                 code: "".into(),
    //             },
    //         )]),
    //     };
    //     let client = exchange::ExchangeClient::new("");
    //     let to_err = |s| format!("不支援的幣別 `{s}`");
    //     let test_cmd = |s| handle_ex_command(command_filter(s).unwrap().unwrap().1, &support, client.clone());
    //     assert_eq!(test_cmd("/ec USD=TWD").await.unwrap(), to_err("USD"));
    //     assert_eq!(test_cmd("/ec 99USD=TWD").await.unwrap(), to_err("USD"));
    // }

    // #[tokio::test]
    // async fn ex_missing_target() {
    //     let support = exchange::Symbols {
    //         symbols: HashMap::from([(
    //             "TWD".into(),
    //             exchange::CurrencyInfo {
    //                 description: "".into(),
    //                 code: "".into(),
    //             },
    //         )]),
    //     };
    //     let client = exchange::ExchangeClient::new("");
    //     let to_err = |s| format!("不支援的幣別 `{s}`");
    //     let test_cmd = |s| handle_ex_command(command_filter(s).unwrap().unwrap().1, &support, client.clone());
    //     assert_eq!(test_cmd("/ec TWD=USD").await.unwrap(), to_err("USD"));
    //     assert_eq!(test_cmd("/ec 99TWD=USD").await.unwrap(), to_err("USD"));
    //     assert_eq!(test_cmd("/ec 55.66TWD=USD").await.unwrap(), to_err("USD"));
    // }

    // #[tokio::test]
    // async fn ex_invalid_format() {
    //     let support = exchange::Symbols {
    //         symbols: HashMap::from([]),
    //     };
    //     let client = exchange::ExchangeClient::new("");
    //     let to_err = || {
    //         "不合法的格式, 應為 `{Amount?}{From}={Target}` 或 `{Amount?}{From} {Target}`"
    //             .to_string()
    //     };
    //     let test_cmd = |s| handle_ex_command(command_filter(s).unwrap().unwrap().1, &support, client.clone());
    //     assert_eq!(test_cmd("/ec ").await.unwrap(), to_err());
    //     assert_eq!(test_cmd("/ec 99").await.unwrap(), to_err());
    //     assert_eq!(test_cmd("/ec 1=TWD").await.unwrap(), to_err());
    // }
}
