use crate::Mode::{ExternalApi, Router};
use actix_web::web::Query;
use actix_web::{get, web, App, HttpServer, Responder};
use reqwest::{Response, StatusCode};
use serde::Deserialize;
use serde_json::json;
use regex::Regex;

enum Mode {
    ExternalApi,
    Router,
}

#[actix_web::main]
async fn main() {
    // reading environment variables
    let auth_token = std::env::var("AUTH")
        .expect("AUTH environment variable is missing.")
        .to_string();
    let domain_data = std::env::var("DOMAIN_DATA")
        .expect("DOMAIN_DATA environment variable is missing.")
        .to_string();
    let mode: Mode = match std::env::var("MODE") {
        Ok(res) => match res.as_str() {
            "external_api" => ExternalApi,
            "router" => Router,
            other => {
                println!("Mode {other} not known. Value defaults to \"external_api\".");
                ExternalApi
            }
        },
        Err(_) => {
            println!("No mode set. Value defaults to \"external_api\".");
            ExternalApi
        }
    };
    let password: Option<String> = match mode {
        ExternalApi => None,
        Router => Some(
            std::env::var("PASSWORD")
                .expect("PASSWORD environment variable is missing. It is needed in router mode.")
                .to_string(),
        ),
    };

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

    match mode {
        // external API mode logic
        ExternalApi => {
            // initialization
            let mut ip_buffer = loop {
                match request_ip_external().await {
                    Ok(res) => break res,
                    Err(err) => {
                        println!("{} Retrying in a minute.", err);
                        wait_minute();
                        continue;
                    }
                }
            };
            println!(
                "🌍 Initial request. Fetched IP Address {} from {} at {}.",
                ip_buffer,
                match mode {
                    ExternalApi => "External API",
                    Router => "Router",
                },
                chrono::Local::now().to_string()
            );
            println!("   🔄 Updating records initially now.");
            handle_domain_data(&domain_data, &auth_token, &ip_buffer).await;

            // loop for external API mode
            loop {
                let nat_ip = loop {
                    match request_ip_external().await {
                        Ok(res) => break res,
                        Err(err) => {
                            println!("{} Retrying in a minute.", err);
                            wait_minute();
                            continue;
                        }
                    }
                };
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
                        Router => "Router",
                    },
                    chrono::Local::now().to_string()
                );
                *&mut ip_buffer = nat_ip.clone();

                println!("   🔄 Updating records now.");
                handle_domain_data(&domain_data, &auth_token, &nat_ip).await;

                // wait 1 minute before next iteration
                wait_minute();
            }
        }

        // router mode logic
        Router => {
            // data management
            let password = password.unwrap();
            let config = web::Data::new(Config {
                auth_token,
                domain_data,
                password,
            });

            // server
            HttpServer::new(move || App::new().app_data(config.clone()).service(index))
                .bind(("0.0.0.0", 3000))
                .unwrap()
                .run()
                .await
                .unwrap();
        }
    }
}

#[get("/")]
async fn index(query: Query<DDNSRequestQuery>, config: web::Data<Config>) -> impl Responder {
    if config.password != query.password {
        return ("Wrong password", StatusCode::UNAUTHORIZED);
    }

    if !check_if_ipv4(&query.ip.as_str()) {
        return ("Not an IPv4", StatusCode::BAD_REQUEST)
    }

    println!(
        "🌍 Fetched NAT IP Address {} from router at {}.",
        query.ip,
        chrono::Local::now().to_string()
    );
    println!("   🔄 Updating records now.");
    handle_domain_data(&config.domain_data, &config.auth_token, &query.ip).await;

    ("Ok", StatusCode::OK)
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

async fn handle_domain_data(domain_data: &Vec<Domain>, auth_token: &str, nat_ip: &str) {
    for domain in domain_data {
        match update_domain(&auth_token, &domain, &nat_ip).await {
            Ok(_) => {
                println!("   ✅ Updated domain with ID {}.", &domain.id)
            }
            // TODO panic here!
            Err(err) => println!("Something went wrong! Error: {err}"),
        };
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
        Err(_err) => {
            Err("External IP request went wrong. There may be no internet connection.".to_string())
        }
    }
}

fn check_if_ipv4(ip: &str) -> bool {
    let regex = Regex::new(r"\d+[.]\d+[.]\d+[.]\d+").unwrap();
    regex.is_match(ip)
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

#[derive(Deserialize)]
struct DDNSRequestQuery {
    ip: String,
    password: String,
}

// config for passing data to web server route
struct Config {
    auth_token: String,
    domain_data: Vec<Domain>,
    password: String,
}
