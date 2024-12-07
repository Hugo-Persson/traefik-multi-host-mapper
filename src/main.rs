use crate::config::ServerConfig;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Command;
use config::{ParsedServer, ServiceConfigTranslate};
use discord::send_to_discord_webhook;
use serde_derive::Serialize;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    sync::Arc,
};
mod config;
mod discord;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let matches = Command::new("MyApp")
        .version("1.0")
        .author("Author Name <author@example.com>")
        .about("Does awesome things")
        .subcommand(Command::new("start-api").about("Starts the API server"))
        .subcommand(Command::new("validate").about("Validates the configuration"))
        .arg_required_else_help(true)
        .get_matches();

    let config_content = fs::read_to_string("config.toml").expect("Failed to read config.toml");

    let config: ServerConfig =
        toml::from_str(&config_content).expect("Failed to parse config.toml");
    match matches.subcommand() {
        Some(("start-api", _)) => start_api(config).await,
        Some(("validate", _)) => validate(config),
        _ => {
            eprintln!("Unknown command");
            Ok(())
        }
    }?;
    Ok(())
}

#[derive(Serialize)]
struct ProviderAPIResponse {
    http: Http,
}

#[derive(Serialize)]
struct Http {
    routers: HashMap<String, Router>,
    services: HashMap<String, Service>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Router {
    middlewares: Vec<String>,
    entry_points: Vec<String>,
    service: String,
    rule: String,
    tls: Tls,
}
#[derive(Serialize)]
struct Tls {
    certresolver: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Service {
    load_balancer: LoadBalancer,
}
#[derive(Serialize)]
struct LoadBalancer {
    servers: Vec<Server>,
}
#[derive(Serialize)]
struct Server {
    url: String,
}

impl ProviderAPIResponse {
    fn from_config(servers: &[ParsedServer]) -> Self {
        let mut routers: HashMap<String, Router> = HashMap::new();
        let mut services: HashMap<String, Service> = HashMap::new();
        for server in servers.iter() {
            for (service_name, service) in server.services.iter().cloned() {
                let router = Router {
                    middlewares: service.middlewares(),
                    entry_points: vec!["websecure".to_string()],
                    service: service_name.clone(),
                    rule: format!("Host(`{}.evercode.se`)", service_name.clone()),
                    tls: Tls {
                        certresolver: "production".to_string(),
                    },
                };

                routers.insert(service_name.clone(), router);
                let server = Server {
                    url: format!("http://{}:{}", server.ip, service.port),
                };
                let load_balancer = LoadBalancer {
                    servers: vec![server],
                };
                let service = Service { load_balancer };
                services.insert(service_name, service);
            }
        }
        ProviderAPIResponse {
            http: Http { routers, services },
        }
    }
}

// Handler for the JSON endpoint
async fn json_endpoint(config: web::Data<Arc<ServiceConfigTranslate>>) -> impl Responder {
    HttpResponse::Ok().json(ProviderAPIResponse::from_config(&config))
}

async fn notify_discord(config: &[ParsedServer]) {
    if let Ok(webhook_url) = env::var("DISCORD_WEBHOOK_URL") {
        let json_obj = ProviderAPIResponse::from_config(config);
        let json_str = serde_json::to_string_pretty(&json_obj).unwrap();
        let msg = format!(
            r#"
    Web API has been started
    JSON Configurations is:

    ```json
    {}
    ```
    "#,
            json_str
        );
        let res = send_to_discord_webhook(webhook_url.as_str(), msg.as_str()).await;
        if let Err(e) = res {
            eprintln!("Failed to send message to Discord: {}", e);
        }
    } else {
        eprintln!("DISCORD_WEBHOOK_URL is not set")
    }
}

async fn start_api(config: ServerConfig) -> std::io::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!("Version: {}", version);
    let config = config.service_map();
    notify_discord(&config).await;
    let data = Arc::new(config);
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listen_url = format!("0.0.0.0:{}", port);

    HttpServer::new(move || {
        let data = Arc::clone(&data); // Clone Arc for each thread
        App::new()
            .app_data(web::Data::new(data))
            .route("/json", web::get().to(json_endpoint))
        // Define the JSON endpoint
    })
    .bind(listen_url)?
    .run()
    .await
    // Add your API starting logic here
}

fn validate(config: ServerConfig) -> std::io::Result<()> {
    let content = config.service_map();

    for server in content {
        let mut ports_used: HashSet<i32> = HashSet::new();
        for (service_name, service) in server.services {
            if ports_used.contains(&service.port) {
                eprintln!(
                    "Port {} is already in use for server {} and service {}",
                    service.port, server.name, service_name
                );
                std::process::exit(1);
            }
            ports_used.insert(service.port);
        }
    }
    println!("Configuration is valid");
    Ok(())
}
