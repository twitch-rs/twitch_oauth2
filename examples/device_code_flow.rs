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

    // Create the builder!
    let mut builder = DeviceUserTokenBuilder::new(client_id, Default::default());

    // Start the device code flow. This will return a code that the user must enter on Twitch
    let code = builder.start(&reqwest).await?;

    println!("Please go to {0}", code.verification_uri);
    println!(
        "Waiting for user to authorize, time left: {0}",
        code.expires_in
    );

    // Finish the auth with wait_for_code, this will return a token if the user has authorized the app
    let mut token = builder.wait_for_code(&reqwest, tokio::time::sleep).await?;

    println!("{:?}\nrefresing token", token);
    // we can also refresh this token, even without a client secret (if the application was created as a public client type in the twitch dashboard).
    token.refresh_token(&reqwest).await?;
    println!("{:?}", token);
    Ok(())
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
}
