use crate::*;

use std::panic::catch_unwind;

#[test]
fn check_url_decode() {
    assert_eq!(
        url_decode("/%"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(url_decode("/%%"), Ok("/%".to_string()));
    assert_eq!(url_decode("/%20a"), Ok("/ a".to_string()));
    assert_eq!(
        url_decode("/%%+a%20+%@"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(
        url_decode("/has%percent.txt"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(url_decode("/%e6"), Ok("/æ".to_string()));
}

#[test]
fn redirects_cleanly() {
    fn fake(host: &str) -> HttpRequest {
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Host".to_string(), host.to_string())],
            method: "GET".to_string(),
            url: "/asset.blob".to_string(),
        }
    }
    fn assert_308(resp: &HttpResponse, expected: &str) {
        assert_eq!(resp.status_code, 308);
        assert!(resp
            .headers
            .iter()
            .any(|(key, value)| key == "Location" && value == expected));
    }
    assert_308(
        &http_request(fake("aaaaa-aa.raw.ic0.app")),
        "https://aaaaa-aa.ic0.app/asset.blob",
    );
    assert_308(
        &http_request(fake("my.http.files.raw.ic0.app")),
        "https://my.http.files.ic0.app/asset.blob",
    );
    assert_308(
        &http_request(fake("raw.ic0.app.raw.ic0.app")),
        "https://raw.ic0.app.ic0.app/asset.blob",
    );
    assert_308(
        &http_request(fake("raw.ic0.app")), // for ?canisterId=
        "https://ic0.app/asset.blob",
    );
    let no_redirect = catch_unwind(|| http_request(fake("raw.ic0.app.ic0.app")).status_code);
    assert!(!matches!(no_redirect, Ok(308)));
    let no_redirect2 = catch_unwind(|| http_request(fake("straw.ic0.app")).status_code);
    assert!(!matches!(no_redirect2, Ok(308)));
}
