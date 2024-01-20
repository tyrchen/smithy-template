use anyhow::Result;
use echo_client_sdk::{config::Token, Client, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::builder()
        .endpoint_url("http://localhost:3000/api")
        .behavior_version_latest()
        .build();
    let client = Client::from_conf(config);

    println!("\n--- Calling echo_message operation without authentication");
    let ret = client.echo_message().message("example").send().await;
    println!("{:?}", ret);

    println!("\n--- Calling signin operation to get a token");

    let ret = client
        .signin()
        .username("test")
        .password("abcd12345")
        .send()
        .await;
    println!("{:?}", ret);

    let token = ret?.token;

    println!("\n-- Calling echo_message operation with authentication");

    let config = Config::builder()
        .endpoint_url("http://localhost:3000/api")
        .bearer_token(Token::new(token, None))
        .behavior_version_latest()
        .build();
    let client = Client::from_conf(config);

    let ret = client.echo_message().message("example").send().await?;

    println!("{:?}", ret);

    Ok(())
}
