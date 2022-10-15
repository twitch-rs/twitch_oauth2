use twitch_oauth2::TwitchToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);

    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let client_id = std::env::var("TWITCH_CLIENT_ID")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::ClientId::new)
        .expect("Please set env: TWITCH_CLIENT_ID or pass client id as an argument");

    let client_secret = std::env::var("TWITCH_CLIENT_SECRET")
        .ok()
        .or_else(|| args.next())
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
