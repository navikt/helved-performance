use dto::*;

mod dto;
mod client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut iverksetting = Iverksetting::new();
    let mut vedtak = Vedtaksdetaljer::new();
    let utbetaling = Utbetaling::new();
    vedtak.add_utbetaling(utbetaling);
    iverksetting.set_vedtak(vedtak);

    let url = "https://utsjekk.intern.dev.nav.no/api/iverksetting/v2";
    let res = client::post(url, &iverksetting).await?;
    println!("{:?}", res);
    // let str = serde_json::to_string(&iverksetting)?;
    // println!("{}", str);

    // let json = r#"{"name":"Bob"}"#;
    // let person: Person = serde_json::from_str(json)?;
    // println!("Hello, {}!", person.name);

    Ok(())
}

