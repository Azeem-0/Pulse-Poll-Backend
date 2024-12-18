// use actix_web::web::Data;

// use webauthn_rs::prelude::*;

// pub fn startup() -> Data<Webauthn> {
//     let rp_id = "localhost";

//     let rp_origin = Url::parse("http://localhost:3000").expect("Invalid URL");
//     let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");

//     let builder = builder.rp_name("Actix-web webauthn-rs");

//     let webauthn = Data::new(builder.build().expect("Invalid configuration"));

//     webauthn
// }

use actix_web::web::Data;
use dotenv::dotenv;
use std::env;
use webauthn_rs::prelude::*;

pub fn startup() -> Data<Webauthn> {
    dotenv().ok();

    let rp_origin_url =
        env::var("RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let rp_origin = Url::parse(&rp_origin_url).expect("Invalid URL");

    let rp_id = env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());

    let builder = WebauthnBuilder::new(&rp_id, &rp_origin).expect("Invalid configuration");

    let builder = builder.rp_name("Actix-web webauthn-rs");

    let webauthn = Data::new(builder.build().expect("Invalid configuration"));

    webauthn
}
