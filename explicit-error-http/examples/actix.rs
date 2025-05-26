use actix_web::{App, HttpResponse, HttpServer, get, http::StatusCode};
use env_logger::Env;
use explicit_error_http::{Error, Fault, HandlerError, HttpError, derive::HandlerErrorHelpers};
use log::{debug, error};
use problem_details::ProblemDetails;
use serde::Serialize;

#[derive(HandlerErrorHelpers)]
struct MyHandlerError(Error);

impl HandlerError for MyHandlerError {
    fn from_error(value: Error) -> Self {
        MyHandlerError(value)
    }

    fn public_fault_response(fault: &Fault) -> impl Serialize {
        #[cfg(debug_assertions)]
        error!("{fault}");

        #[cfg(not(debug_assertions))]
        error!("{}", serde_json::json!(fault));

        ProblemDetails::new()
            .with_type(http::Uri::from_static("/errors/internal-server-error"))
            .with_title("Internal server error")
    }

    fn error(&self) -> &Error {
        &self.0
    }

    fn domain_response(error: &explicit_error_http::DomainError) -> impl Serialize {
        if error.output.http_status_code.as_u16() < 500 {
            debug!("{error}");
        } else {
            error!("{error}");
        }

        error
    }
}

#[get("/domain")]
async fn domain_error() -> Result<HttpResponse, MyHandlerError> {
    service::operation_on_entity()?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/fault")]
async fn fault_error() -> Result<HttpResponse, MyHandlerError> {
    service::fetch_entity()?;

    Err(HttpError {
        http_status_code: StatusCode::FORBIDDEN,
        public: Box::new(""),
        context: None,
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| App::new().service(domain_error).service(fault_error))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

mod service {
    use crate::db;
    use actix_web::http::StatusCode;
    use explicit_error_http::{HttpError, Result, derive::HttpError, prelude::*};
    use problem_details::ProblemDetails;

    #[derive(HttpError, Debug)]
    pub enum MyDomainError {
        EntityNotFound(String),
        Validation,
    }

    impl From<&MyDomainError> for HttpError {
        fn from(value: &MyDomainError) -> Self {
            match value {
                MyDomainError::EntityNotFound(name) => HttpError {
                    http_status_code: StatusCode::NOT_FOUND,
                    public: Box::new(
                        ProblemDetails::new()
                            .with_type(http::Uri::from_static("/errors/entity/not-found"))
                            .with_title("Article not found.")
                            .with_detail(format!("Name: {name}")),
                    ),
                    context: None,
                },
                MyDomainError::Validation => HttpError {
                    http_status_code: StatusCode::BAD_REQUEST,
                    public: Box::new(
                        ProblemDetails::new()
                            .with_type(http::Uri::from_static("/errors/entity/validation"))
                            .with_title("Data provided for the operation is incorrect."),
                    ),
                    context: None,
                },
            }
        }
    }

    #[derive(HttpError, Debug)]
    pub struct SubDomainError {
        x99: &'static str,
    }

    impl From<&SubDomainError> for HttpError {
        fn from(value: &SubDomainError) -> Self {
            HttpError {
                http_status_code: StatusCode::NOT_FOUND,
                public: Box::new(
                    ProblemDetails::new()
                        .with_type(http::Uri::from_static("/errors/subdomain/x99"))
                        .with_title(value.x99),
                ),
                context: Some("Some usefull info to debug".to_string()),
            }
        }
    }

    pub fn operation_on_entity() -> Result<()> {
        if 0 > 1 {
            Err(MyDomainError::Validation)?
        }

        if "X99".len() < 12 {
            subdomain()?;
        }

        db::fetch_entity().map_err_or_fault(|e| match e {
            sqlx::Error::RowNotFound => Ok(MyDomainError::EntityNotFound("Optimus".to_string())),
            e => Err(e),
        })?;

        Ok(())
    }

    pub fn fetch_entity() -> Result<()> {
        db::timed_out()
            .fault()
            .with_context("Usefull info to help debug")?;

        Ok(())
    }

    pub fn subdomain() -> Result<()> {
        Err(SubDomainError { x99: "X99" })?
    }
}

mod db {
    use sqlx::Error;

    pub fn fetch_entity() -> Result<(), Error> {
        Err(Error::RowNotFound)
    }

    pub fn timed_out() -> Result<(), Error> {
        Err(Error::PoolTimedOut)
    }
}
