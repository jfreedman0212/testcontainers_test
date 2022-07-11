use crate::{
    db_handle::DbHandle,
    domain::{errors::*, Login, User},
};
use actix_files::NamedFile;
use actix_web::{get, http::header::ContentType, post, web, HttpResponse, Responder};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use handlebars::Handlebars;
use serde::Serialize;
use snafu::ResultExt;

#[get("/")]
#[tracing::instrument(name = "Navigating to the login page", skip(hb))]
pub(crate) async fn login_page(
    hb: web::Data<Handlebars<'static>>,
) -> Result<HttpResponse, ServerError> {
    let html =
        tokio::task::spawn_blocking(move || hb.render("index", &LoginModel { message: None }))
            .await
            .context(JoinSnafu)?
            .context(TemplateRenderingSnafu {
                template_name: String::from("index"),
            })?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html))
}

#[post("/")]
#[tracing::instrument(name = "Logging In", skip(input, db_handle), fields(username = %input.username()))]
pub(crate) async fn login(
    input: web::Form<Login>,
    db_handle: web::Data<DbHandle>,
    hb: web::Data<Handlebars<'static>>,
) -> Result<HttpResponse, ServerError> {
    let username = input.username().to_string();
    let user_option = db_handle
        .query_row(
            "SELECT Id, Username, PasswordHash FROM User WHERE Username = ?1",
            [username],
            |row| {
                let id: i64 = row.get(0)?;
                let username: String = row.get(1)?;
                let password_hash: String = row.get(2)?;
                Ok((User::new(id, username), password_hash))
            },
        )
        .await?;
    let argon = Argon2::default();
    match user_option {
        Some((user, password_hash)) => {
            let matches = tokio::task::spawn_blocking(move || {
                argon
                    .verify_password(
                        input.password().as_bytes(),
                        &PasswordHash::new(&password_hash).unwrap(),
                    )
                    .is_ok()
            })
            .await
            .context(JoinSnafu)?;
            if matches {
                Ok(HttpResponse::Ok().json(user))
            } else {
                let html = tokio::task::spawn_blocking(move || {
                    hb.render(
                        "index",
                        &LoginModel {
                            message: Some(MessageToClient::InvalidUsernameOrPassword),
                        },
                    )
                })
                .await
                .context(JoinSnafu)?
                .context(TemplateRenderingSnafu {
                    template_name: String::from("index"),
                })?;
                Ok(HttpResponse::BadRequest()
                    .content_type(ContentType::html())
                    .body(html))
            }
        }
        None => {
            let html = tokio::task::spawn_blocking(move || {
                hb.render(
                    "index",
                    &LoginModel {
                        message: Some(MessageToClient::InvalidUsernameOrPassword),
                    },
                )
            })
            .await
            .context(JoinSnafu)?
            .context(TemplateRenderingSnafu {
                template_name: String::from("index"),
            })?;
            Ok(HttpResponse::BadRequest()
                .content_type(ContentType::html())
                .body(html))
        }
    }
}

#[derive(Serialize)]
enum MessageToClient {
    InvalidUsernameOrPassword,
}

#[derive(Serialize)]
struct LoginModel {
    message: Option<MessageToClient>,
}
