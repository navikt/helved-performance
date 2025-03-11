use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder};
use log::{info, warn};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use crate::env_or_default;
use chrono::{DateTime, NaiveDate, Utc};
use rand::Rng;

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

const PERSONIDENT: [&str; 3] = [
    "18439049363",
    "12460271795",
    "20416818623",
];

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Iverksetting {
    sak_id: String,
    behandling_id: String,
    iverksetting_id: Option<String>,
    personident: Personident,
    vedtak: Vedtaksdetaljer,
    forrige_iverksetting: Option<ForrigeIverksetting>,
}

impl Iverksetting {
    pub fn new() -> Iverksetting {
        Iverksetting {
            sak_id: rand::random::<u32>().to_string(),
            behandling_id: rand::random::<u32>().to_string(),
            iverksetting_id: None,
            personident: Personident::default(),
            vedtak: Vedtaksdetaljer::new(),
            forrige_iverksetting: None,
        }
    }

    pub fn set_vedtak(&mut self, vedtak: Vedtaksdetaljer) {
        self.vedtak = vedtak;
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vedtaksdetaljer {
    vedtakstidspunkt: DateTime<Utc>,
    saksbehandler_id: String,
    beslutter_id: String,
    utbetalinger: Vec<Utbetaling>,
}

impl Vedtaksdetaljer {
    pub fn new() -> Vedtaksdetaljer {
        Vedtaksdetaljer {
            vedtakstidspunkt: Utc::now(),
            saksbehandler_id: "A123456".to_string(),
            beslutter_id: "B234567".to_string(),
            utbetalinger: vec![],
        }
    }

    pub fn add_utbetaling(&mut self, utbetaling: Utbetaling) {
        self.utbetalinger.push(utbetaling);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Utbetaling {
    beløp: u32,
    satstype: SatsType,
    fra_og_med_dato: NaiveDate,
    til_og_med_dato: NaiveDate,
    stønadsdata: Stønadsdata,
}

impl Utbetaling {
    pub fn new() -> Self {
        Utbetaling {
            beløp: 700,
            satstype: SatsType::DAGLIG,
            fra_og_med_dato: NaiveDate::from_ymd_opt(2021, 1, 1).expect("ugyldig dato"),
            til_og_med_dato: NaiveDate::from_ymd_opt(2021, 1, 7).expect("ugyldig dato"),
            stønadsdata: Stønadsdata::Dagpenger {
                stønadstype: "DAGPENGER_ARBEIDSSØKER_ORDINÆR".to_string(),
                ferietillegg: Some(Ferietillegg::ORDINÆR),
                meldekort_id: "123456".to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Stønadsdata {
    #[serde(rename_all = "camelCase")]
    Dagpenger {
        stønadstype: String,
        ferietillegg: Option<Ferietillegg>,
        meldekort_id: String,
    },

    #[serde(rename_all = "camelCase")]
    Tiltakspenger {
        stønadstype: String,
        barnetillegg: bool,
        brukers_nav_kontor: String,
        meldekort_id: String,
    },

    #[serde(rename_all = "camelCase")]
    Tilleggsstønader {
        stønadstype: String,
        brukers_nav_kontor: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForrigeIverksetting {
    behandling_id: String,
    iverksetting_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Personident {
    verdi: String,
}

impl Default for Personident {
    fn default() -> Self {
        let mut rng = rand::rng();
        Personident {
            verdi: PERSONIDENT[rng.random_range(1..=3)].to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum SatsType {
    DAGLIG,
    MÅNEDLIG,
    ENGANGS,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Ferietillegg {
    ORDINÆR,
    AVDØD,
}
