use crate::helpers::{spawn_test_app, TestApp};
use reqwest;
use tokio;

#[tokio::test]
async fn health_check() {
    let TestApp { address } = spawn_test_app().await;
    let response = reqwest::get(format!("http://{}/health_check", address))
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response was not successful"
    );
}
