use anyhow::Context;
use twitch_oauth2::tokens::UserTokenBuilder;

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
        .context("Please set env: TWITCH_CLIENT_ID or pass as first argument")?;

    let client_secret = std::env::var("TWITCH_CLIENT_SECRET")
        .ok()
        .or_else(|| args.next())
        .map(twitch_oauth2::ClientSecret::new)
        .context("Please set env: TWITCH_CLIENT_SECRET or pass as second argument")?;

    let redirect_url = std::env::var("TWITCH_REDIRECT_URL")
        .ok()
        .or_else(|| args.next())
        .map(|r| twitch_oauth2::url::Url::parse(&r))
        .context("Please set env: TWITCH_REDIRECT_URL or pass as third argument")??;

    let mut builder =
        UserTokenBuilder::new(client_id, client_secret, redirect_url).force_verify(true);

    let (url, _) = builder.generate_url();

    println!("Go to this page: {}", url);

    let input = rpassword::prompt_password(
        "Paste in the resulting adress after authenticating (input hidden): ",
    )?;

    let u = twitch_oauth2::url::Url::parse(&input).context("when parsing the input as a URL")?;

    let map: std::collections::HashMap<_, _> = u.query_pairs().collect();

    match (map.get("state"), map.get("code")) {
        (Some(state), Some(code)) => {
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
