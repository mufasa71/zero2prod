#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use sqlx::PgPool;
    use zero2prod::routes::get_subscription;

    #[sqlx::test(fixtures("subscriptions"))]
    async fn test_index_get(db_pool: PgPool) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db_pool.clone()))
                .route("/", web::get().to(get_subscription)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/?email=ursula_le_guin@gmail.com")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
