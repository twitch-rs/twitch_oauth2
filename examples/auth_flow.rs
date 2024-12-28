//! This is an example of the Authorization code grant flow using `twitch_oauth2`
//!
//! See https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
//!
//! See also the `device_code_flow` example for possibly easier integration.

use anyhow::Context;
use twitch_oauth2::tokens::UserTokenBuilder;

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
        .context("Please set env: TWITCH_CLIENT_ID or pass as first argument")?;

    // Grab the client secret, convert to a `ClientSecret` with the `new` method.
    let client_secret = get_env_or_arg("TWITCH_CLIENT_SECRET", &mut args)
        .map(twitch_oauth2::ClientSecret::new)
        .context("Please set env: TWITCH_CLIENT_SECRET or pass as second argument")?;

    // Grab the redirect URL, this has to be set verbatim in the developer console: https://dev.twitch.tv/console/apps/
    let redirect_url = get_env_or_arg("TWITCH_REDIRECT_URL", &mut args)
        .map(|r| twitch_oauth2::url::Url::parse(&r))
        .context("Please set env: TWITCH_REDIRECT_URL or pass as third argument")??;

    // Create the builder!
    let mut builder =
        UserTokenBuilder::new(client_id, client_secret, redirect_url).force_verify(true);

    // Generate the URL, this is the url that the user should visit to authenticate.
    let (url, _) = builder.generate_url();

    println!("Go to this page: {}", url);

    let input = rpassword::prompt_password(
        "Paste in the resulting adress after authenticating (input hidden): ",
    )?;

    let u = twitch_oauth2::url::Url::parse(&input).context("when parsing the input as a URL")?;

    // Grab the query parameters "state" and "code" from the url the user was redirected to.
    let map: std::collections::HashMap<_, _> = u.query_pairs().collect();

    match (map.get("state"), map.get("code")) {
        (Some(state), Some(code)) => {
            // Finish the builder with `get_user_token`
            let token = builder.get_user_token(&reqwest, state, code).await?;
            println!("Got token: {:?}", token);
        }
        _ => match (map.get("error"), map.get("error_description")) {
            (std::option::Option::Some(error), std::option::Option::Some(error_description)) => {
                anyhow::bail!(
                    "twitch errored with error: {} - {}",
                    error,
                    error_description
                );
            }
            _ => anyhow::bail!("invalid url passed"),
        },
    }
    Ok(())
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
}
