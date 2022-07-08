use crate::helpers::spawn_test_app;
use reqwest;
use testcontainers::clients::Cli;
use tokio;

#[tokio::test]
async fn health_check() {
    let docker = Cli::default();
    let test_app = spawn_test_app(&docker);
    let response = reqwest::get(format!("http://{}/health_check", test_app.address))
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "Response was not successful"
    );
}
