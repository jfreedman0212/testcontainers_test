use reqwest;
use std::net::TcpListener;
use testcontainers_test::run;
use tokio;

#[tokio::test]
async fn greet() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let _ = tokio::spawn(async move {
        run(listener).unwrap().await.unwrap();
    });
    let response = reqwest::get(format!("http://{}/health_check", address))
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response was not successful"
    );
}
