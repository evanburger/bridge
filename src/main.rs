use std::error::Error;


const CONFIG_FILE_PATH: &str = "./.config.json";
// const TOKEN_FILE_PATH: &str = "./token.txt";  // FUTURE



mod cli {
    use crate::Error;

    use serde;
    use serde_json;


    #[derive(serde::Deserialize)]
    pub struct Config {
        pub app_name: String,
        pub base_path: String,
        pub config_sync_resource_name: String,
        pub config_sync_field_name: String,
        pub origin: String,
        pub resources: serde_json::Value,
    }

    impl Config {
        pub fn from_file<P: AsRef<std::path::Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
            let file = std::fs::File::open(file_path)?;
            let reader = std::io::BufReader::new(file);
            let config: Self = serde_json::from_reader(reader)?;
            Ok(config)
        }
    }
}




mod sdk {
    use crate::Error;

    use reqwest::blocking::Client;


    const COOKIE_NAME: &str = "directus_session_token";


    enum HttpMethod {
        Get,
        Post,
        Patch,
        Delete,
    }

    #[allow(dead_code)]
    pub enum ResponseFormat {
        Json,
        Text,
    }

    #[derive(serde::Deserialize)]
    struct JsonResult {
        data: serde_json::Value,
    }

    pub struct DirectusSdk {
        api_path: String,
        config_sync_field_name: String,
        reqwest_client: Client,  // TODO: make into generic http Client
    }
    
    impl DirectusSdk {
        pub fn from_cookie(origin: String, base_path: String, token: String, config_sync_field_name: String) -> Self {
            let session_cookie = format!("{}={}", COOKIE_NAME, token);
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("Cookies", session_cookie.parse().unwrap());
            let reqwest_client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();
            Self {
                api_path: format!("{}{}", origin, base_path),
                config_sync_field_name: config_sync_field_name,
                reqwest_client: reqwest_client,
            }
        }
    
        fn _response(&self, url: String, data: Option<&String>, method: HttpMethod) -> Result<reqwest::blocking::Response, Box<dyn Error>> {
            Ok(match method {
                HttpMethod::Get | HttpMethod::Delete => {
                    self.reqwest_client.get(url).send()?
                },
                HttpMethod::Post | HttpMethod::Patch => {
                    let data_value: serde_json::Value = serde_json::from_str(data.unwrap()).unwrap_or_else(|_| {
                        eprintln!("error: invalid JSON data input for operation");
                        std::process::exit(1);
                    });
                    let generic_response = match method {
                        HttpMethod::Post => {
                            self.reqwest_client.post(url).json(&data_value).send()?
                        },
                        HttpMethod::Patch => {
                            self.reqwest_client.patch(url).json(&data_value).send()?
                        },
                        _ => {
                            eprintln!("error: invalid HTTP method");
                            std::process::exit(1);
                        },
                    };
                    generic_response
                },
            })
        }

        fn _path(&self, resource: &String, id: Option<&String>) -> String {
            match id {
                Some(id) => format!("{}/{}/{}", self.api_path, resource, id),
                None => format!("{}/{}", self.api_path, resource),
            }
        }

        pub fn create(&self, resource: &String, data: &String) -> Result<String, Box<dyn Error>> {
            let generic_response = self._response(self._path(resource, None), Some(data), HttpMethod::Post)?;
            if generic_response.status().is_success() {
                Ok(String::from("created successfully"))
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "error: failed to post",
                )))
            }
        }
    
        pub fn read(&self, resource: &String, id: Option<&String>, format: Option<ResponseFormat>, is_sync: bool) -> Result<String, Box<dyn Error>> {
            let generic_response = self._response(self._path(resource, id), None, HttpMethod::Get)?;
            if generic_response.status().is_success() {
                let response = match format {
                    None | Some(ResponseFormat::Json) => {
                        let json_result: JsonResult = serde_json::from_str(
                            generic_response.text()?.as_str()
                        ).unwrap();
                        if is_sync {
                            json_result.data.get(&self.config_sync_field_name).unwrap().to_string()
                        } else {
                            json_result.data.to_string()
                        }
                    },
                    Some(ResponseFormat::Text) => generic_response.text()?,
                };
                Ok(response)
            } else {
                eprintln!("error: failed to get");
                std::process::exit(1);
            }
        }
    
        pub fn update(&self, resource: &String, id: &String, data: &String) -> Result<String, Box<dyn Error>> {
            let generic_response = self._response(self._path(resource, Some(id)), Some(data), HttpMethod::Patch)?;
            if generic_response.status().is_success() {
                Ok(String::from("updated successfully"))
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "error: failed to put",
                )))
            }
        }
    
        pub fn delete(&self, resource: &String, id: &String) -> Result<String, Box<dyn Error>> {
            let generic_response = self._response(self._path(resource, Some(id)), None, HttpMethod::Delete)?;
            if generic_response.status().is_success() {
                Ok(String::from("deleted successfully"))
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "error: failed to delete",
                )))
            }
        }
    }
}



fn json_keys<'a>(json_value: &'a serde_json::Value) -> Option<serde_json::map::Keys<'a>> {
    match json_value {
        serde_json::Value::Object(object) => Some(object.keys()),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = cli::Config::from_file(CONFIG_FILE_PATH)?;
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);  // The first arg is the program name, which is not needed.
    let top_level_command = args.get(0);
    // let session_token: String = std::fs::read_to_string(TOKEN_FILE_PATH)?.trim().to_string();  // FUTURE
    let session_token = String::from("test");  // DEBUG
    let directus = sdk::DirectusSdk::from_cookie(
        config.origin,
        config.base_path,
        session_token,
        config.config_sync_field_name,
    );
    if !top_level_command.is_none() {
        match top_level_command.unwrap() as &str {
            "auth" => {
                eprintln!("auth command not implemented yet");
                std::process::exit(1);
            },
            "ls" => {
                for resource in json_keys(&config.resources).unwrap() {
                    println!("{}", resource);
                }
                return Ok(());
            },
            "inspect" => {
                let resource_name = args.get(1);
                if !resource_name.is_some() {
                    eprintln!("error: resource name required for inspect command");
                    std::process::exit(1);
                }
                println!(
                    "{}",
                    &config.resources.get(resource_name.unwrap())
                        .unwrap_or_else(|| {
                            eprintln!("error: unknown resource");
                            std::process::exit(1);
                        })
                );
                return Ok(());
            },
            "sync" => {
                println!("{}", directus.read(&config.config_sync_resource_name, Some(&config.app_name), None, true)?);
                return Ok(());
            },
            "create" | "read" | "update" | "delete" => {},
            _ => {
                eprintln!("error: unknown command");
                std::process::exit(1);
            },
        }
    } else {
        eprintln!("error: <operation> positional arg required");
        std::process::exit(1);
    }
    if args.len() < 2 {
        eprintln!("error: <operation> and <resource> positional args required");
        std::process::exit(1);
    }
    if args.len() > 4 {
        eprintln!("error: too many args");
        std::process::exit(1);
    }
    let available_resources: Vec<&String> = json_keys(&config.resources)
        .unwrap()
        .collect();
    let chosen_resource = &args[1];
    if !available_resources.contains(&chosen_resource) {
        eprintln!("error: resource not available");
        std::process::exit(1);
    }
    let resource_config = config.resources.get(chosen_resource).unwrap();
    let available_resource_ops: Vec<&String> = json_keys(resource_config.get("ops").unwrap())
        .unwrap()
        .collect();
    let chosen_resource_op = &args[0];
    if !available_resource_ops.contains(&chosen_resource_op) {
        eprintln!("error: operation not available for that resource");
        std::process::exit(1);
    }
    let response = match chosen_resource_op as &str {
        "create" => {
            let data = args.get(2);
            if !data.is_some() {
                eprintln!("error: data required for create operation");
                std::process::exit(1);
            }
            directus.create(chosen_resource, data.unwrap())?
        },
        "read" => {
            let id = args.get(2);
            directus.read(chosen_resource, id, None, false)?
        },
        "update" => {
            let id = args.get(2);
            if !id.is_some() {
                eprintln!("error: id required for update operation");
                std::process::exit(1);
            }
            let data = args.get(2);
            if !data.is_some() {
                eprintln!("error: data required for update operation");
                std::process::exit(1);
            }
            directus.update(chosen_resource, id.unwrap(), data.unwrap())?
        },
        "delete" => {
            let id = args.get(2);
            if !id.is_some() {
                eprintln!("error: id required for delete operation");
                std::process::exit(1);
            }
            directus.delete(chosen_resource, id.unwrap())?
        },
        _ => String::from("error: unknown operation"),
    };
    println!("{}", response);
    Ok(())
}
