use twitch_oauth2::{client, TwitchToken};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);

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

    let scopes = std::env::var("CLIENT_SCOPES")
        .ok()
        .map(|s| s.split(' ').map(|s| s.to_string()).collect::<Vec<_>>())
        .or_else(|| Some(args.collect::<Vec<_>>()))
        .map(|v| v.into_iter().map(twitch_oauth2::Scope::from).collect())
        .expect("Please set env: CLIENT_SCOPES or pass client secret as an argument");

    let token = twitch_oauth2::AppAccessToken::get_app_access_token(
        &surf::Client::new(),
        client_id,
        client_secret,
        scopes,
    )
    .await?;
    println!("{:?}", token);
    dbg!(token.is_elapsed());
    Ok(())
}
