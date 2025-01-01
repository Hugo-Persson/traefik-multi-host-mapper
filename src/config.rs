use serde_derive::Deserialize;
use std::collections::HashMap;
use std::vec;
use toml::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server: HashMap<String, Value>, // Nested structure to hold dynamic configurations
}
#[derive(Debug)]
pub struct ParsedServer {
    pub ip: String,
    pub name: String,
    pub services: Vec<(String, ServiceConfig)>,
}
pub type ServiceConfigTranslate = Vec<ParsedServer>;
impl ServerConfig {
    pub fn service_map(&self) -> ServiceConfigTranslate {
        let mut service_map: ServiceConfigTranslate = Vec::new();
        let ignore_services = ["ip", "mac"];

        for (server, value) in self.server.iter() {
            let ip = value.get("ip").unwrap().as_str().unwrap();

            let mut services: Vec<(String, ServiceConfig)> = Vec::new();
            for (service_name, service) in value.as_table().unwrap().iter() {
                if ignore_services.contains(&service_name.to_owned().as_str()) {
                    continue;
                }
                let toml_string = toml::to_string(service).expect("Failed to serialize");

                let config: ServiceConfig =
                    toml::from_str(&toml_string).expect("Failed to deserialize");
                services.push((service_name.to_string(), config));
            }

            let data = ParsedServer {
                ip: ip.to_string(),
                name: server.to_owned(),
                services,
            };
            service_map.push(data);
        }
        service_map
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub port: i32,
    #[serde(default)]
    pub authelia: bool,

    #[serde(default)]
    pub authentik: bool,

    #[serde(default)]
    pub https: bool,

    #[serde(default)]
    pub extra_domains: Vec<String>,
}

impl ServiceConfig {
    pub fn middlewares(&self) -> Vec<String> {
        if self.authelia {
            vec!["authelia@docker".to_string()]
        } else if self.authentik {
            vec!["authentik@file".to_string()]
        } else {
            vec![]
        }
    }
}
