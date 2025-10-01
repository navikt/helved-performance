pub mod aap {
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
        utbetalt_beløp: u32
    }
}

pub enum Utbetaling<'a> {
    Aap(&'a aap::Utbetaling),
    Dp(&'a dp::Utbetaling),
    Ts(&'a ts::Utbetaling),
    Tp(&'a tp::Utbetaling),
}

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

pub mod ts
{
    use chrono::{DateTime, NaiveDate, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Utbetaling 
    {
        pub dryrun: bool,
        id: Uuid,
        sak_id: String,
        behandling_id: String,
        personident: String,
        stønad: Stønadtype,
        vedtakstidspunkt: DateTime<Utc>,
        periodetype: Periodetype,
        perioder: Vec<Periode>,
        bruk_fagområde_tillst: bool,
        saksbehandler: Option<String>,
        beslutter: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Periode {
        fom: NaiveDate,
        tom: NaiveDate,
        beløp: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Periodetype {
        Dag,
        Ukedag,
        Mnd,
        EnGang,
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Stønadtype {
        TILSYN_BARN_ENSLIG_FORSØRGER,
        TILSYN_BARN_AAP,
        TILSYN_BARN_ETTERLATTE,
        LÆREMIDLER_ENSLIG_FORSØRGER,
        LÆREMIDLER_AAP,
        LÆREMIDLER_ETTERLATTE,
        BOUTGIFTER_AAP,
        BOUTGIFTER_ENSLIG_FORSØRGER,
        BOUTGIFTER_ETTERLATTE,
        DAGLIG_REISE_ENSLIG_FORSØRGET,
        DAGLIG_REISE_AAP,
        DAGLIG_REISE_ETTERLATTE,
        REISE_TIL_SAMLING_ENSLIG_FORSØRGER,
        REISE_TIL_SAMLING_AAP,
        REISE_TIL_SAMLING_ETTERLATTE,
        REISE_OPPSTART_ENSLIG_FORSØRGET,
        REISE_OPPSTART_AAP,
        REISE_OPPSTART_ETTERLATTE,
        REIS_ARBEID_ENSLIG_FORSØRGER,
        REIS_ARBEID_AAP,
        REIS_ARBEID_ETTERLATTE,
        FLYTTING_ENSLIG_FORSØRGER,
        FLYTTING_AAP,
        FLYTTING_ETTERLATTE,
    }
}

pub mod tp
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
        personident: String,
        stønad: Stønadtype,
        vedtakstidspunkt: DateTime<Utc>,
        perioder: Vec<Periode>,
        saksbehandler: Option<String>,
        beslutter: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Periode {
        meldeperiode: String,
        fom: NaiveDate,
        tom: NaiveDate,
        beløp: u32,
    }

    #[allow(non_camel_case_types)]
    #[allow(clippy::upper_case_acronyms)]
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Stønadtype {
        ARBEIDSFORBEREDENDE_TRENING,
        ARBEIDSRETTET_REHABILITERING,
        ARBEIDSTRENING,
        AVKLARING,
        DIGITAL_JOBBKLUBB,
        ENKELTPLASS_AMO,
        ENKELTPLASS_VGS_OG_HØYERE_YRKESFAG,
        FORSØK_OPPLÆRING_LENGRE_VARIGHET,
        GRUPPE_AMO,
        GRUPPE_VGS_OG_HØYERE_YRKESFAG,
        HØYERE_UTDANNING,
        INDIVIDUELL_JOBBSTØTTE,
        INDIVIDUELL_KARRIERESTØTTE_UNG,
        JOBBKLUBB,
        OPPFØLGING,
        UTVIDET_OPPFØLGING_I_NAV,
        UTVIDET_OPPFØLGING_I_OPPLÆRING,
        ARBEIDSFORBEREDENDE_TRENING_BARN,
        ARBEIDSRETTET_REHABILITERING_BARN,
        ARBEIDSTRENING_BARN,
        AVKLARING_BARN,
        DIGITAL_JOBBKLUBB_BARN,
        ENKELTPLASS_AMO_BARN,
        ENKELTPLASS_VGS_OG_HØYERE_YRKESFAG_BARN,
        FORSØK_OPPLÆRING_LENGRE_VARIGHET_BARN,
        GRUPPE_AMO_BARN,
        GRUPPE_VGS_OG_HØYERE_YRKESFAG_BARN,
        HØYERE_UTDANNING_BARN,
        INDIVIDUELL_JOBBSTØTTE_BARN,
        INDIVIDUELL_KARRIERESTØTTE_UNG_BARN,
        JOBBKLUBB_BARN,
        OPPFØLGING_BARN,
        UTVIDET_OPPFØLGING_I_NAV_BARN,
        UTVIDET_OPPFØLGING_I_OPPLÆRING_BARN,
    }
}

pub mod status 
{
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Reply {
        pub status: Status,
        pub error: Option<Error>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Error {
        pub status_code: u16,
        pub msg: String,
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

