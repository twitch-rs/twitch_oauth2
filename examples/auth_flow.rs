use anyhow::Context;
use twitch_oauth2::tokens::UserTokenBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv(); // Eat error
    let mut args = std::env::args().skip(1);
    let mut builder = UserTokenBuilder::new(
        std::env::var("TWITCH_CLIENT_ID")
            .ok()
            .or_else(|| args.next())
            .map(twitch_oauth2::ClientId::new)
            .context("Please set env: TWITCH_CLIENT_ID or pass as first argument")?,
        std::env::var("TWITCH_CLIENT_SECRET")
            .ok()
            .or_else(|| args.next())
            .map(twitch_oauth2::ClientSecret::new)
            .context("Please set env: TWITCH_CLIENT_SECRET or pass as second argument")?,
        std::env::var("TWITCH_REDIRECT_URL")
            .ok()
            .or_else(|| args.next())
            .map(|r| twitch_oauth2::url::Url::parse(&r))
            .context("Please set env: TWITCH_REDIRECT_URL or pass as third argument")??,
    )
    .force_verify(true);

    let (url, _) = builder.generate_url();

    println!("Go to this page: {}", url);

    let input = rpassword::prompt_password_stdout(
        "Paste in the resulting adress after authenticating (input hidden): ",
    )?;

    let u = twitch_oauth2::url::Url::parse(&input).context("when parsing the input as a URL")?;

    let map: std::collections::HashMap<_, _> = u.query_pairs().collect();

    match (map.get("state"), map.get("code")) {
        (Some(state), Some(code)) => {
            let token = builder
                .get_user_token(
                    &reqwest::Client::builder()
                        .redirect(reqwest::redirect::Policy::none())
                        .build()?,
                    state,
                    code,
                )
                .await?;
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
