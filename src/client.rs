#![allow(dead_code)]

use reqwest::Response;

use crate::Iverksetting;

pub async fn post(url: &str, iverksetting: &Iverksetting) -> anyhow::Result<Response> {
    let client = reqwest::Client::new();

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", "bearer blærhp")
        .json(iverksetting)
        .send()
        .await?;

    Ok(res)
}

fn get_auth_token() -> String {
    let client_id = env_or_default("AZURE_APP_CLIENT_ID", "lololo");


    todo!()
}

fn env_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}
