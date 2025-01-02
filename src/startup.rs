use actix_web::web::Data;
use dotenv::dotenv;
use log::{error, info};
use std::env;
use webauthn_rs::prelude::*;

pub fn startup() -> Result<Data<Webauthn>, String> {
    dotenv().ok();
    info!("Loading environment variables...");

    // Retrieve RP_ORIGIN from environment variables or use default
    let rp_origin_url = env::var("RP_ORIGIN").unwrap_or_else(|_| {
        error!("RP_ORIGIN not found in environment, using default value");
        "http://localhost:3000".to_string()
    });

    let rp_origin = Url::parse(&rp_origin_url).map_err(|e| {
        let error_message = format!("Invalid RP_ORIGIN URL '{}': {}", rp_origin_url, e);
        error!("{}", error_message);
        error_message
    })?;

    // Retrieve RP_ID from environment variables or use default
    let rp_id = env::var("RP_ID").unwrap_or_else(|_| {
        error!("RP_ID not found in environment, using default value");
        "localhost".to_string()
    });

    info!(
        "Initializing Webauthn with RP_ID: {} and RP_ORIGIN: {}",
        rp_id, rp_origin
    );

    let builder = WebauthnBuilder::new(&rp_id, &rp_origin).map_err(|e| {
        let error_message = format!("Failed to initialize WebauthnBuilder: {}", e);
        error!("{}", error_message);
        error_message
    })?;

    let builder = builder.rp_name("Actix-web webauthn-rs");

    let webauthn = builder.build().map_err(|e| {
        let error_message = format!("Failed to build Webauthn instance: {}", e);
        error!("{}", error_message);
        error_message
    })?;

    info!("Webauthn initialized successfully");
    Ok(Data::new(webauthn))
}
