use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::web::Data;

use webauthn_rs::prelude::*;

use crate::config::config::LoginStateStore;

pub fn startup() -> (Data<Webauthn>, Data<LoginStateStore>) {
    let rp_id = "localhost";

    let rp_origin = Url::parse("http://localhost:3000").expect("Invalid URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");

    let builder = builder.rp_name("polling application backend rust actix-web");

    let webauthn = Data::new(builder.build().expect("Invalid configuration"));

    let login_store = Data::new(Arc::new(Mutex::new(HashMap::new())));

    (webauthn, login_store)
}
