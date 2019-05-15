#![cfg(feature = "actix")]

extern crate actix_web;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use actix_web::{FromRequest, HttpResponse};
use qs::actix::{QsQuery, QsQueryConfig};
use serde::de::Error;

fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(&s).map_err(|_| D::Error::custom("could not parse string"))
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Query {
    foo: u64,
    bars: Vec<u64>,
    #[serde(flatten)]
    common: CommonParams,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct CommonParams {
    #[serde(deserialize_with = "from_str")]
    limit: u64,
    #[serde(deserialize_with = "from_str")]
    offset: u64,
    #[serde(deserialize_with = "from_str")]
    remaining: bool,
}

#[test]
fn test_default_error_handler() {
    let req = TestRequest::with_uri("/test").to_srv_request();
    let (req, mut pl) = req.into_parts();

    let e = QsQuery::<Query>::from_request(&req, &mut pl).unwrap_err();
    assert_eq!(
        e.as_response_error().error_response().status(),
        StatusCode::BAD_REQUEST
    );
}

#[test]
fn test_custom_error_handler() {
    let req = TestRequest::with_uri("/test")
        .data(QsQueryConfig::default().error_handler(|e, _| {
            let resp = HttpResponse::UnprocessableEntity().finish();
            InternalError::from_response(e, resp).into()
        }))
        .to_srv_request();

    let (req, mut pl) = req.into_parts();
    let query = QsQuery::<Query>::from_request(&req, &mut pl);

    assert!(query.is_err());
    assert_eq!(
        query
            .unwrap_err()
            .as_response_error()
            .error_response()
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[test]
fn test_composite_querystring_extractor() {
    let req = TestRequest::with_uri(
        "/test?foo=1&bars[]=0&bars[]=1&limit=100&offset=50&remaining=true",
    )
    .to_srv_request();
    let (req, mut pl) = req.into_parts();

    let s = QsQuery::<Query>::from_request(&req, &mut pl).unwrap();
    assert_eq!(s.foo, 1);
    assert_eq!(s.bars, vec![0, 1]);
    assert_eq!(s.common.limit, 100);
    assert_eq!(s.common.offset, 50);
    assert_eq!(s.common.remaining, true);
}
