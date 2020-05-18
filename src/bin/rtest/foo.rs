
mod foo {

    use reqwest;
    use serde::{Deserialize, Serialize};
    use std::error;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Foo {
        pub name: String,
        pub foo_id: u32,
        pub  pid: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FooListingResponse {
        total: u32,
        page_size: u32,
        page: u32,
        list: Vec<Foo>,
    }

    #[derive(Debug)]
    pub struct FooConnection {
        http_client: reqwest::blocking::Client,
        access_token: String,
        api_host: String,
    }

    impl FooConnection {
        pub fn new(auth_token: String, ims_env: String) -> FooConnection {
            FooConnection {
                http_client: reqwest::blocking::Client::new(),
                api_host: String::from("https://api.example.com"),
                access_token: auth_token,
            }
        }

        pub fn get_data(&mut self) -> Result<Vec<Foo>, Box<dyn error::Error>> {
            let mut data_sources: Vec<Foo> = Vec::new();

            Ok(data_sources)
        }
    }
}