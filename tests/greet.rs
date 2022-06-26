use reqwest;
use std::net::TcpListener;
use testcontainers_test::run;
use tokio;

#[tokio::test]
async fn greet() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let _ = tokio::spawn(async {
        run(listener).unwrap().await.unwrap();
    });
    let response = reqwest::get(format!("http://{}/hello/josh", address))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(response, "Hello josh!");
}
