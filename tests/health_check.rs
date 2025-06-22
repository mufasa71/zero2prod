use serde_json::Value;
use sqlx::PgPool;
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{configuration::get_configuration, run};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[sqlx::test]
async fn health_check_works(db_pool: PgPool) {
    let test_app = spawn_app(db_pool).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[sqlx::test]
async fn subscriptions_returns_a_200_for_valid_form_data(db_pool: PgPool) {
    let test_app = spawn_app(db_pool.clone()).await;
    let client = reqwest::Client::new();
    let body = "user_name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/api/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!(
        "SELECT email, user_name FROM subscriptions WHERE email = $1",
        "ursula_le_guin@gmail.com"
    )
    .fetch_one(&db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.user_name, "le guin");
}

#[sqlx::test(fixtures("subscriptions"))]
async fn subscriptions_returns_a_200_and_result(db_pool: PgPool) {
    let test_app = spawn_app(db_pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/subscriptions", &test_app.address))
        .query(&[("email", "ursula_le_guin@gmail.com")])
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let subscription: Value =
        serde_json::from_str(&response.text().await.expect("Failed to get body"))
            .expect("Failed to parse JSON");

    assert_eq!(subscription["email"], "ursula_le_guin@gmail.com");
}

#[sqlx::test]
async fn subscriptions_returns_a_404(db_pool: PgPool) {
    let test_app = spawn_app(db_pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/subscriptions", &test_app.address))
        .query(&[("email", "ursula_le_guin@gmail.com")])
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());
}

#[sqlx::test]
async fn subscriptions_returns_a_400_when_data_is_missing(db_pool: PgPool) {
    let test_app = spawn_app(db_pool).await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/api/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app(db_pool: PgPool) -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut config = get_configuration().expect("Failed to read configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let server = run(listener, db_pool.clone()).expect("Failed to bind address");

    tokio::spawn(server);

    TestApp { address, db_pool }
}
