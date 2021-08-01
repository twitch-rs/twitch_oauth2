use twitch_oauth2::TwitchToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error

    let mut args = std::env::args().skip(1);
    std::env::var("TWITCH_OAUTH2_URL")
        .ok()
        .or_else(|| args.next())
        .map(|t| std::env::set_var("TWITCH_OAUTH2_URL", &t))
        .expect("Please set env: TWITCH_OAUTH2_URL or pass url as first argument");

    let client_id = std::env::var("MOCK_CLIENT_ID")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::ClientId::new)
        .expect("Please set env: MOCK_CLIENT_ID or pass client id as an argument");

    let client_secret = std::env::var("MOCK_CLIENT_SECRET")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::ClientSecret::new)
        .expect("Please set env: MOCK_CLIENT_SECRET or pass client secret as an argument");

    let token = twitch_oauth2::AppAccessToken::get_app_access_token(
        &reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?,
        client_id,
        client_secret,
        vec![],
    )
    .await?;
    println!("token");
    Ok(())
}
