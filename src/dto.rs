#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

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
        Personident {
            verdi: PERSONIDENT[rand::random::<usize>() % 10].to_string(),
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
