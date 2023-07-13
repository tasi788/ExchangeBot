use grammers_client;
use grammers_client::{Client, Config, InputMessage, SignInError, Update};
use grammers_session::Session;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{self, BufRead as _, Write as _};
use tokio::{runtime, task};

use exchange_bot as lib;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;
type Results<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SESSION_FILE: &str = "exchangebot.session";

async fn get_support() -> Option<lib::Symbols> {
    let url = "https://api.exchangerate.host/symbols";
    match reqwest::get(url)
        .await
        .expect("req error")
        .json::<lib::Symbols>()
        .await
    {
        Ok(t) => Some(t),
        _ => None,
    }
    // Some(res)
}

async fn get_exchange(from: &str, target: &str, value: &str) -> Option<lib::RespResult> {
    let url = format!(
        "https://api.exchangerate.host/convert?from={from}&to={target}&amount={amount}",
        from = from,
        target = target,
        amount = value
    );
    match reqwest::get(url)
        .await
        .expect("req error")
        .json::<lib::RespResult>()
        .await
    {
        Ok(t) => Some(t),
        _ => None,
    }
}

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

async fn handle_update(client: Client, update: Update, support: lib::Symbols) -> Result {
    match update {
        Update::NewMessage(message) if message.outgoing() && message.text().starts_with("/ex") => {
            message.edit("查詢中").await?;
            let split = message.text().split(" ").collect::<Vec<&str>>();
            // let chat = message.chat();
            match split.clone().len() {
                x => {
                    if x != 2 {
                        message.edit("指令錯誤").await.unwrap();
                        // client.send_message(&chat, "指令錯誤").await.unwrap();
                    } else {
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
                        let text = match RE.captures(&split[1]) {
                            Some(caps)
                                if !support.symbols.contains_key(&caps["from"].to_uppercase()) =>
                            {
                                format!("不支援的幣別 `{from}`", from = &caps["from"])
                            }
                            Some(caps)
                                if !support
                                    .symbols
                                    .contains_key(&caps["target"].to_uppercase()) =>
                            {
                                format!("不支援的幣別 `{target}`", target = &caps["target"])
                            }
                            Some(caps) => {
                                let amount = &caps["amount"];
                                let from = &caps["from"];
                                let target = &caps["target"];
                                get_exchange(from, target, amount)
                                    .await
                                    .map(|r| {
                                        format!(
                                        "`{source}` `{from}` 對 `{target}` 的匯率為 `{amount:.2}` ",
                                        source = amount.to_uppercase(),
                                        from = from.to_uppercase(),
                                        target = target.to_uppercase(),
                                        amount = r.result
                                    )
                                    })
                                    .unwrap()
                            }
                            None => {
                                "Invalid format, Expected `{Amount?}{From}={Target}`".to_string()
                            }
                        };
                        message.edit(InputMessage::markdown(text)).await.unwrap();
                    }
                }
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
        // let curr = ;
        task::spawn(async move {
            match handle_update(handle, update, get_support().await.unwrap()).await {
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
