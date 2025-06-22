use actix_web::{
    Either, HttpRequest, HttpResponse, Responder,
    body::BoxBody,
    http::{Error, header::ContentType},
    web,
};
use derive_more::derive::Display;
use log::info;
use serde::Serialize;
use sqlx::{
    FromRow,
    postgres::PgPool,
    types::{
        Uuid,
        chrono::{DateTime, Utc},
    },
};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    user_name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Info {
    email: String,
}

#[derive(Debug, FromRow, Serialize, Display)]
#[display("{email}")]
pub struct Subscription {
    id: Uuid,
    email: String,
    user_name: String,
    subscribed_at: DateTime<Utc>,
}

impl Responder for Subscription {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let rec = sqlx::query!(
        "INSERT INTO subscriptions ( email, user_name ) VALUES ( $1, $2 )",
        form.email,
        form.user_name
    )
    .execute(db_pool.as_ref())
    .await;

    if let Ok(_rec) = rec {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::BadRequest().finish()
}

type SubscriptionResult = Either<HttpResponse, Result<Subscription, Error>>;

pub async fn get_subscription(
    info: web::Query<Info>,
    db_pool: web::Data<PgPool>,
) -> SubscriptionResult {
    let subscription =
        sqlx::query_as::<_, Subscription>("SELECT * FROM subscriptions WHERE email = $1")
            .bind(&info.email)
            .fetch_one(db_pool.get_ref())
            .await;

    match subscription {
        Ok(subscription) => Either::Right(Ok(subscription)),
        Err(e) => {
            info!("{}", e);

            match e {
                sqlx::Error::RowNotFound => Either::Left(HttpResponse::NotFound().finish()),
                _ => Either::Left(HttpResponse::InternalServerError().finish()),
            }
        }
    }
}

pub fn subscriptions_config(cfg: &mut web::ServiceConfig) {
    let resource = web::resource("/subscriptions")
        .get(get_subscription)
        .post(subscribe);

    cfg.service(resource);
}
