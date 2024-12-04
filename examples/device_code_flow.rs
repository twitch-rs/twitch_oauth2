//! Example of how to create a user token using device code flow.
//! This example only works properly on a public client type.
use twitch_oauth2::{DeviceUserTokenBuilder, TwitchToken, UserToken};

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

    let mut builder = DeviceUserTokenBuilder::new(client_id, Default::default());

    // Start the device code flow. This will return a code that the user must enter on
    let code = builder.start(&reqwest).await?;

    println!("Please go to {0}", code.verification_uri);
    println!("Waiting for user to authorize");

    // Finish the auth with finish, this will return a token if the user has authorized the app
    let mut finish = builder.finish(&reqwest).await;
    // on the error type for `finish`, there's a convenience function to check if the request is pending.
    while finish.as_ref().is_err_and(|e| e.is_pending()) {
        // wait a bit for next check
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        finish = builder.finish(&reqwest).await;
    }
    let token = finish?;

    println!("{:?}", token);
    dbg!(token.is_elapsed());
    // we can also refresh this token, even without a client secret.
    token.refresh_token(&reqwest).await?;
    println!("{:?}", token);
    Ok(())
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
}
