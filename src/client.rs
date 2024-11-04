#![allow(dead_code)]

use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder};
use log::{info, warn};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use crate::{dto::*, env_or_default};

pub async fn iverksett() -> HttpResponse {
    let mut iverksetting = Iverksetting::new();
    let mut vedtak = Vedtaksdetaljer::new();
    let utbetaling = Utbetaling::new();
    vedtak.add_utbetaling(utbetaling);
    iverksetting.set_vedtak(vedtak);

    info!("iverksetter: {:?} ..", iverksetting);

    let url = "http://utsjekk/api/iverksetting/v2";

    match post(url, &iverksetting).await {
        Ok(res) => {
            info!("response: {:?}", &res);
            let status = &res.status().as_u16();
            let status = StatusCode::from_u16(status.to_owned()).expect("Invalid status code");
            let body = res.text().await.unwrap();
            HttpResponseBuilder::new(status).body(body)
        }
        Err(err) => {
            warn!("response: {:?}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}

async fn post<T: Serialize>(url: &str, body: &T) -> anyhow::Result<Response> {
    let client = reqwest::Client::new();
    let token = get_auth_token().await?;

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("bearer {token}"))
        .json(body)
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

#[derive(Serialize, Deserialize, Debug)]
struct AccessTokenBody {
    access_token: String,
}

