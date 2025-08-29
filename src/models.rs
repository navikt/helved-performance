pub mod dp
{
    use chrono::{DateTime, NaiveDate, Utc};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Utbetaling 
    {
        pub dryrun: bool,
        sak_id: String,
        behandling_id: String,
        ident: String,
        utbetalinger: Vec<Utbetalingsdag>,
        vedtakstidspunktet: DateTime<Utc>,
        saksbehandler: Option<String>,
        beslutter: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Utbetalingsdag {
        meldeperiode: String,
        dato: NaiveDate,
        sats: u32,
        utbetalt_beløp: u32,
        rettighetstype: Rettighetstype,
        utbetalingstype: Utbetalingstype,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Utbetalingstype {
        DagpengerFerietillegg,
        Dagpenger,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Rettighetstype {
        Ordinær,
        Permittering,
        PermitteringFiskeindustrien,
        EØS,
    }
}

pub mod status 
{
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Reply {
        pub status: Status,
        error: Option<Error>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Error {
        status_code: i32,
        msg: String,
        doc: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Status {
        Ok,
        Feilet,
        Mottatt,
        HosOppdrag,
    }
}

pub mod dryrun
{
    use serde::{Deserialize, Serialize};
    use chrono::NaiveDate;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Simulering {
        perioder: Vec<Periode>
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Periode {
        fom: NaiveDate,
        tom: NaiveDate,
        utbetalinger: Vec<Utbetaling>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Utbetaling {
        fagsystem: String,
        sak_id: String,
        utbetales_til: String,
        stønadstype: String,
        tidligere_utbetalt: i32,
        nytt_beløp: i32,
    }
}

