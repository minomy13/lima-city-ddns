use crate::Mode::ExternalApi;
use reqwest::{Response, StatusCode};
use serde::Deserialize;
use serde_json::json;

enum Mode {
    ExternalApi,
}

#[tokio::main]
async fn main() {
    // reading environment variables
    let auth_token = std::env::var("AUTH").unwrap().to_string();
    let domain_data = std::env::var("DOMAIN_DATA").unwrap().to_string();
    // used for future feature
    let mode: Mode = ExternalApi;

    // parsing data from environment variables
    let domain_data: Vec<Domain> = domain_data
        .split(";")
        .into_iter()
        .map(|element| {
            let split = element.split(":");
            let mut record_ids = vec![];
            split
                .clone()
                .last()
                .unwrap()
                .split(",")
                .for_each(|record_id| {
                    record_ids.push(Record {
                        id: record_id.to_string(),
                    });
                });
            Domain {
                id: split.collect::<Vec<&str>>().get(0).unwrap().to_string(),
                records: record_ids,
            }
        })
        .collect();

    // initialization
    let mut ip_buffer = request_ip_external().await.unwrap();
    println!(
        "🌍 Initial request. Fetched IP Address {} from {} at {}.",
        ip_buffer,
        match mode {
            ExternalApi => "External API",
        },
        chrono::Local::now().to_string()
    );
    println!("   🔄 Updating records initially now.");
    for domain in &domain_data {
        match update_domain(&auth_token, &domain, &ip_buffer).await {
            Ok(_) => {
                println!("   ✅ Updated domain with ID {}.", &domain.id)
            }
            Err(err) => println!("Something went wrong! Error: {err}"),
        };
    }

    // loop for external API mode
    if matches!(mode, ExternalApi) {
        loop {
            let nat_ip = request_ip_external().await.unwrap();
            if nat_ip.eq(&ip_buffer) {
                // wait 1 minute before next iteration
                wait_minute();
                continue;
            }

            println!(
                "🌍 Fetched NAT IP Address {} from {} at {}.",
                nat_ip,
                match mode {
                    ExternalApi => "External API",
                },
                chrono::Local::now().to_string()
            );
            *&mut ip_buffer = nat_ip.clone();

            println!("   🔄 Updating records now.");
            for domain in &domain_data {
                match update_domain(&auth_token, &domain, &nat_ip).await {
                    Ok(_) => {
                        println!("   ✅ Updated domain with ID {}.", &domain.id)
                    }
                    Err(err) => println!("Something went wrong! Error: {err}"),
                };
            }

            // wait 1 minute before next iteration
            wait_minute();
        }
    }
}

async fn update_record(
    auth: &str,
    domain_id: &str,
    record_id: &str,
    new_content: &str,
) -> Result<Response, String> {
    let client = reqwest::Client::new();
    match client
        .put(format!(
            "https://www.lima-city.de/usercp/domains/{domain_id}/records/{record_id}"
        ))
        .basic_auth("api", Some(auth))
        .json(&json!({
          "nameserver_record": {
            "content": new_content,
          },
        }))
        .send()
        .await
    {
        Ok(res) => match res.status() {
            StatusCode::OK => Ok(res),
            other => Err(format!("Request failed with status code {other}")),
        },
        Err(_err) => Err("Request went wrong.".to_string()),
    }
}

async fn update_domain(auth: &str, domain: &Domain, new_content: &str) -> Result<(), String> {
    for record in &domain.records {
        update_record(auth, &domain.id, &record.id, new_content).await?;
    }
    Ok(())
}

async fn request_ip_external() -> Result<String, String> {
    let client = reqwest::Client::new();
    match client
        .get("https://api64.ipify.org?format=json")
        .send()
        .await
    {
        Ok(res) => Ok(res.json::<IpAPIResponse>().await.unwrap().ip),
        Err(_err) => Err("External IP request went wrong.".to_string()),
    }
}

fn wait_minute() {
    std::thread::sleep(std::time::Duration::from_secs(60))
}

struct Domain {
    id: String,
    records: Vec<Record>,
}

struct Record {
    id: String,
}

#[derive(Deserialize)]
struct IpAPIResponse {
    ip: String,
}
