use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json;
use sha2::Sha256;
use std::error;
use std::fs::File;
use std::io::prelude::*;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub key: String,
    pub passphrase: String,
    pub secret: String,
    pub token_lifetime: String,
}

#[derive(Debug)]
pub struct Order {
    pub created: String,
    pub fee: f32,
    pub price: f32,
    pub quantity: f32,
    pub cost: f32,
}

pub fn read_file_contents(path: String) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

impl Account {
    pub fn new(config: Account) -> Self {
        Self {
            id: config.id,
            key: config.key,
            passphrase: config.passphrase,
            secret: config.secret,
            token_lifetime: config.token_lifetime,
        }
    }

    pub fn account_token(&self) -> Result<String> {
        let timestamp = Utc::now().timestamp_millis();
        let path: String = format!("/accounts/{}/api_keys/{}/tokens", self.id, self.key);
        let body: String = serde_json::json!({
            "expiresIn": self.token_lifetime
        })
        .to_string();

        let sig = self.account_token_signature(&timestamp, &path, &body);
        let client = Client::new();
        let r_body = client
            .post(format!("https://api.nbx.com{path}"))
            .header(
                "authorization",
                format!("NBX-HMAC-SHA256 {}:{}", self.passphrase, sig?),
            )
            .header("x-nbx-timestamp", timestamp)
            .body(body.clone())
            .send()?
            .text()?;
        let v: serde_json::Value =
            serde_json::from_str(r_body.as_str()).expect("Could not deserialize JSON");
        Ok(v["token"].to_string().trim_matches('"').to_string())
    }

    fn account_token_signature(
        &self,
        timestamp: &i64,
        path: &String,
        body: &String,
    ) -> Result<String> {
        let mut mac: Hmac<Sha256> =
            Hmac::new_from_slice(base64::decode(self.secret.clone())?.as_slice())?;
        mac.update(format!("{timestamp}POST{path}{body}").as_bytes());
        Ok(base64::encode(mac.finalize().into_bytes()))
    }

    pub fn account_asset_by_id(&self, asset: &String, token: &String) -> Result<f32> {
        let client = Client::new();
        let r_body = client
            .get(format!(
                "https://api.nbx.com/accounts/{}/assets/{}",
                self.id, asset
            ))
            .header("authorization", format!("Bearer {}", token))
            .send()?
            .text()?;

        let v: serde_json::Value = serde_json::from_str(&r_body)?;
        Ok(v["balance"]["available"]
            .to_string()
            .trim_matches('"')
            .parse()?)
    }
}

impl Order {
    pub fn create(
        account: &Account,
        token: &String,
        side: &String,
        ticker: &String,
        amount: f64,
    ) -> Result<bool> {
        let market = format!("{}-NOK", ticker);

        let body: String = match side.to_uppercase().as_str() {
            "BUY" => serde_json::json!({
                "market": market,
                "side": "BUY",
                "quantity": "1",
                "execution": {
                    "type": "MARKET",
                    "timeInForce": {
                        "type": "IMMEDIATE_OR_CANCEL"
                    },
                "freeze": {
                    "type": "AMOUNT",
                    "value": amount.to_string()
                }
            }
            }),
            "SELL" => serde_json::json!({
                "market": market,
                "side": "SELL",
                "quantity": amount.to_string(),
                "execution": {
                    "type": "MARKET",
                    "timeInForce": {
                        "type": "IMMEDIATE_OR_CANCEL"
                    }
                }
            }),
            _ => serde_json::json!({}),
        }
        .to_string();

        let client = Client::new();
        let response = client
            .post(format!(
                "https://api.nbx.com/accounts/{}/orders",
                account.id
            ))
            .header("authorization", format!("Bearer {}", token))
            .body(body.clone()) // Cloned for logging
            .send()?;

        let status = response.status();
        let response_body = response.text()?; // Get the response body

        if status.is_success() {
            Ok(true)
        } else {
            // Log the response body on error
            println!("Failed to create order. Response: {}", response_body);
            Ok(false)
        }
    }

    pub fn details_by_last_id(account: &Account, token: &String) -> Order {
        let client = Client::new();
        let req = client
            .get(format!(
                "https://api.nbx.com/accounts/{}/orders",
                account.id
            ))
            .header("Authorization", format!("Bearer {}", token));
        let res = req.send().unwrap().text().unwrap();
        let v: serde_json::Value = serde_json::from_str(&res).unwrap();
        let last = v.as_array().unwrap()[0].to_string();
        let v: serde_json::Value = serde_json::from_str(&last).unwrap();
        let id = v["id"].as_str().unwrap().to_string();

        Order::details_by_id(&account, &token, &id).unwrap()
    }

    pub fn details_by_id(account: &Account, token: &String, order_id: &String) -> Result<Order> {
        let client = Client::new();
        let r_text = client
            .get(format!(
                "https://api.nbx.com/accounts/{}/orders/{}",
                account.id, order_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()?
            .text()?;

        let v: serde_json::Value = serde_json::from_str(&r_text)?;
        let fills = v["fills"].to_string();
        let v: serde_json::Value = serde_json::from_str(&fills)?;
        let created = v[0]["createdAt"].as_str().unwrap();
        let fee: f32 = v[0]["fee"].as_str().unwrap().parse()?;
        let price: f32 = v[0]["price"].as_str().unwrap().parse()?;
        let quantity: f32 = v[0]["quantity"].as_str().unwrap().parse()?;

        let details = Order {
            created: created.to_string(),
            fee,
            price,
            quantity,
            cost: price * quantity,
        };

        Ok(details)
    }
}
