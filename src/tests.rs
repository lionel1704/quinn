extern crate untrusted;

use rustls::internal::pemfile;

use std::{fs::File, io::{BufReader, Read}};
use std::sync::Arc;

use crypto::Secret;
use tls::{ClientTls, ServerTls};
use types::{Endpoint, Side};

use self::untrusted::Input;

use webpki;

#[test]
fn test_handshake() {
    let mut c = client_endpoint();
    let initial = c.initial("example.com");

    let mut s = server_endpoint(initial.conn_id().unwrap());
    let server_hello = s.handle_handshake(&initial).unwrap();
    assert!(c.handle_handshake(&server_hello).is_some());
}

fn server_endpoint(hs_cid: u64) -> Endpoint<ServerTls> {
    let certs = {
        let f = File::open("certs/server.chain").expect("cannot open 'certs/server.chain'");
        let mut reader = BufReader::new(f);
        pemfile::certs(&mut reader).expect("cannot read certificates")
    };

    let keys = {
        let f = File::open("certs/server.rsa").expect("cannot open 'certs/server.rsa'");
        let mut reader = BufReader::new(f);
        pemfile::rsa_private_keys(&mut reader).expect("cannot read private keys")
    };

    let tls_config = Arc::new(ServerTls::build_config(certs, keys[0].clone()));
    Endpoint::new(
        ServerTls::with_config(&tls_config),
        Side::Server,
        Some(Secret::Handshake(hs_cid)),
    )
}

fn client_endpoint() -> Endpoint<ClientTls> {
    let tls = {
        let mut f = File::open("certs/ca.der").expect("cannot open 'certs/ca.der'");
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).expect("error while reading");

        let anchor =
            webpki::trust_anchor_util::cert_der_as_trust_anchor(Input::from(&bytes)).unwrap();
        let anchor_vec = vec![anchor];
        let config = ClientTls::build_config(Some(&webpki::TLSServerTrustAnchors(&anchor_vec)));
        ClientTls::with_config(config)
    };

    Endpoint::new(tls, Side::Client, None)
}