#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_derive;

extern crate gotham;
extern crate mime;

use cqrs_es::{AggregateError, Command};
use std::collections::HashMap;

use crate::aggregate::BankAccount;
use commands::OpenAccount;
use events::BankAccountEvent;
use postgres::{Connection, TlsMode};
use postgres_es::{GenericQueryRepository, PostgresCqrs};
use queries::{BankAccountQuery, SimpleLoggingQueryProcessor};
use serde::de::DeserializeOwned;

mod aggregate;
mod application;
mod commands;
mod events;
mod queries;

use futures::prelude::*;
use gotham::hyper::{body, Body, StatusCode};
use std::pin::Pin;

use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::create_empty_response;
use gotham::helpers::http::response::create_response;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryIdPathExtractor {
    query_id: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct AccountCommandPathExtractor {
    command_type: String,
    aggregate_id: String,
}

fn main() {
    let addr = "127.0.0.1:3030";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

fn account_query_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    println!("called");
    let query_id = QueryIdPathExtractor::take_from(&mut state).query_id;
    let query_repo = AccountQuery::new("account_query", db_connection());
    println!("{}", query_id);
    let f = match query_repo.load(query_id) {
        None => {
            let res = create_empty_response(&state, StatusCode::NOT_FOUND);
            future::ok((state, res))
        }
        Some(query) => {
            let body = serde_json::to_string(&query).unwrap();
            let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, body);
            // res.headers = std_headers();
            future::ok((state, res))
        }
    };

    f.boxed()
}

fn account_command_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(valid_body) => {
            let extractor = AccountCommandPathExtractor::take_from(&mut state);
            let command_type = extractor.command_type;
            let aggregate_id = extractor.aggregate_id;
            let payload = String::from_utf8(valid_body.to_vec()).unwrap();
            let result = match command_type.as_str() {
                "openAccount" => process_command::<OpenAccount>(aggregate_id.as_str(), payload),
                _ => {
                    let res = create_empty_response(&state, StatusCode::NOT_FOUND);
                    return future::ok((state, res));
                }
            };

            match result {
                Ok(_) => {
                    let res = create_empty_response(&state, StatusCode::NO_CONTENT);
                    future::ok((state, res))
                }
                Err(err) => {
                    let err_payload = match &err {
                        AggregateError::UserError(e) => serde_json::to_string(e).unwrap(),
                        AggregateError::TechnicalError(e) => e.clone(),
                    };
                    let res = create_response(
                        &state,
                        StatusCode::BAD_REQUEST,
                        mime::APPLICATION_JSON,
                        err_payload,
                    );
                    future::ok((state, res))
                }
            }
        }
        Err(e) => future::err((state, e.into_handler_error())),
    });

    f.boxed()
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/accounts/:query_id")
            .with_path_extractor::<QueryIdPathExtractor>()
            .to(account_query_handler);
        route
            .post("/commands/:command_type/:aggregate_id")
            .with_path_extractor::<AccountCommandPathExtractor>()
            .to(account_command_handler);
    })
}

pub fn process_command<T>(aggregate_id: &str, payload: String) -> Result<(), AggregateError>
where
    T: Command<BankAccount, BankAccountEvent> + DeserializeOwned,
{
    let payload = match serde_json::from_str::<T>(payload.as_str()) {
        Ok(payload) => payload,
        Err(err) => {
            return Err(AggregateError::TechnicalError(err.to_string()));
        }
    };

    let cqrs = cqrs_framework();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
    cqrs.execute_with_metadata(aggregate_id, payload, metadata)
}

// fn std_headers() -> Headers {
//     let mut headers = Headers::new();
//     let content_type = iron::headers::ContentType::json();
//     headers.set(content_type);
//     headers
// }

type AccountQuery = GenericQueryRepository<BankAccountQuery, BankAccount, BankAccountEvent>;

pub fn cqrs_framework() -> PostgresCqrs<BankAccount, BankAccountEvent> {
    let simple_query = SimpleLoggingQueryProcessor {};
    let mut account_query_processor = AccountQuery::new("account_query", db_connection());
    account_query_processor.with_error_handler(Box::new(|e| println!("{}", e)));
    postgres_es::postgres_cqrs(
        db_connection(),
        vec![Box::new(simple_query), Box::new(account_query_processor)],
    )
}

pub fn db_connection() -> Connection {
    Connection::connect(
        "postgres://demo_user:demo_pass@localhost:5432/demo",
        TlsMode::None,
    )
    .unwrap()
}
