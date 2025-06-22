pub mod configuration;
pub mod routes;

use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use routes::health_check;
use sqlx::PgPool;

use crate::routes::subscriptions_config;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        let logger = Logger::default();
        let api_scope = web::scope("/api").app_data(db_pool.clone());

        App::new()
            .wrap(logger)
            .service(api_scope.configure(subscriptions_config))
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
