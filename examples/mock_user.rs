//! Example of how to create a mock user token
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);

    // Setup the http client to use with the library.
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    get_env_or_arg("TWITCH_OAUTH2_URL", &mut args)
        .map(|t| std::env::set_var("TWITCH_OAUTH2_URL", &t))
        .expect("Please set env: TWITCH_OAUTH2_URL or pass url as first argument");

    let client_id = get_env_or_arg("MOCK_CLIENT_ID", &mut args)
        .map(twitch_oauth2::ClientId::new)
        .expect("Please set env: MOCK_CLIENT_ID or pass client id as an argument");

    let client_secret = get_env_or_arg("MOCK_CLIENT_SECRET", &mut args)
        .map(twitch_oauth2::ClientSecret::new)
        .expect("Please set env: MOCK_CLIENT_SECRET or pass client secret as an argument");

    let user_id = get_env_or_arg("MOCK_USER_ID", &mut args)
        .expect("Please set env: MOCK_USER_ID or pass user_id as an argument");

    // Using a mock token from twitch-cli mock is very similar to using a regular token, however you need to call `UserToken::mock_token` instead of `UserToken::from_existing` etc.
    let token =
        twitch_oauth2::UserToken::mock_token(&reqwest, client_id, client_secret, user_id, vec![])
            .await?;
    println!(
        "token retrieved: {} - {:?}",
        token.access_token.secret(),
        token
    );
    Ok(())
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
}
