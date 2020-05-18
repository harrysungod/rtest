use reqwest;
use reqwest::Method;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
struct Ip {
    ip: String,
}

fn main() {

    let http_client = reqwest::blocking::Client::new();
    let url = "https://api.ipify.org/?format=json";
    let req = http_client.request(Method::GET, url);

    let req = req.build().unwrap();
    println!("{:?}", req);
    println!("{:?}", req.body());

    let resp = http_client.execute(req).unwrap();
    println!("{:?}", resp);
    // println!("{:?}", resp.bytes());

    let ip: Ip = resp.json().expect("can't get ip");
    println!("{:?}", ip)
}