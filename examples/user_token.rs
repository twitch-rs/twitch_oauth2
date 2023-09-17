//! Example of how to create a UserToken from an existing token.
//!
//! See the auth_flow example for how to create a token from scratch.
use twitch_oauth2::TwitchToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);

    // Setup the http client to use with the library.
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    // Grab the token, convert to a `AccessToken` with the `new` method.
    let user_token = get_env_or_arg("TWITCH_TOKEN", &mut args)
        .map(twitch_oauth2::AccessToken::new)
        .expect("Please set env: TWITCH_TOKEN or pass token as first argument");

    // Grab refresh token, not necessarily required.
    let refresh_token =
        get_env_or_arg("TWITCH_REFRESH_TOKEN", &mut args).map(twitch_oauth2::RefreshToken::new);

    // Grab the client secret, not necessarily required, unless you have a refresh token and want to refresh the token with `UserToken::refresh`.
    let client_secret =
        get_env_or_arg("TWITCH_CLIENT_SECRET", &mut args).map(twitch_oauth2::ClientSecret::new);

    let token =
        twitch_oauth2::UserToken::from_existing(&reqwest, user_token, refresh_token, client_secret)
            .await?;

    println!("{:?}", token);
    dbg!(token.is_elapsed());
    Ok(())
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
}
