use actix_web::web::Data;

use webauthn_rs::prelude::*;

pub fn startup() -> Data<Webauthn> {
    let rp_id = "localhost";

    let rp_origin = Url::parse("http://localhost:3000").expect("Invalid URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");

    let builder = builder.rp_name("Actix-web webauthn-rs");

    let webauthn = Data::new(builder.build().expect("Invalid configuration"));

    webauthn
}
