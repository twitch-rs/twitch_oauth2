//! Example of how to create a app access token using client credentials
use twitch_oauth2::TwitchToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);

    // Setup the http client to use with the library.
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    // Grab the client id, convert to a `ClientId` with the `new` method.
    let client_id = get_env_or_arg("TWITCH_CLIENT_ID", &mut args)
        .map(twitch_oauth2::ClientId::new)
        .expect("Please set env: TWITCH_CLIENT_ID or pass client id as an argument");

    // Grab the client secret, convert to a `ClientSecret` with the `new` method.
    let client_secret = get_env_or_arg("TWITCH_CLIENT_SECRET", &mut args)
        .map(twitch_oauth2::ClientSecret::new)
        .expect("Please set env: TWITCH_CLIENT_SECRET or pass client secret as an argument");

    // Get the app access token
    let token = twitch_oauth2::AppAccessToken::get_app_access_token(
        &reqwest,
        client_id,
        client_secret,
        vec![],
    )
    .await?;

    println!("{:?}", token);
    dbg!(token.is_elapsed());
    Ok(())
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
}
