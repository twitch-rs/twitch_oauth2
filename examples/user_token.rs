use twitch_oauth2::TwitchToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);

    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let user_token = std::env::var("TWITCH_TOKEN")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::AccessToken::new)
        .expect("Please set env: TWITCH_TOKEN or pass token as first argument");

    let refresh_token = std::env::var("TWITCH_REFRESH_TOKEN")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::RefreshToken::new);

    let client_secret = std::env::var("TWITCH_CLIENT_SECRET")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::ClientSecret::new);

    let token =
        twitch_oauth2::UserToken::from_existing(&reqwest, user_token, refresh_token, client_secret)
            .await?;

    println!("{:?}", token);
    dbg!(token.is_elapsed());
    Ok(())
}
