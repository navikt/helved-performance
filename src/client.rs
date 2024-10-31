#![allow(dead_code)]

use reqwest::Response;
use serde::{Deserialize, Serialize};
use crate::Iverksetting;

pub async fn post(url: &str, iverksetting: &Iverksetting) -> anyhow::Result<Response> {
    let client = reqwest::Client::new();
    let token = get_auth_token().await?;

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("bearer {token}"))
        .json(iverksetting)
        .send()
        .await?;

    Ok(res)
}

async fn get_auth_token() -> anyhow::Result<String> {
    let client_id = env_or_default("AZURE_APP_CLIENT_ID", "");
    let client_secret = env_or_default("AZURE_APP_CLIENT_SECRET", "");
    let url = env_or_default("AZURE_OPENID_CONFIG_TOKEN_ENDPOINT", "");
    let body = format!("client_id={}&client_secret={}&scope=api://dev-gcp.helved.utsjekk/.default&grant_type=client_credentials", client_id, client_secret);

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;
    let json = res.json::<AccessTokenBody>().await?;

    Ok(json.access_token)
}

fn env_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct AccessTokenBody {
    access_token: String,
}