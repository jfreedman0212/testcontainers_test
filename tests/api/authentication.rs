use reqwest::Client;
use testcontainers_test::domain::{Login, UserInput};

use crate::helpers::{spawn_test_app, TestApp};

#[tokio::test]
async fn create_user_success() {
    let TestApp { address } = spawn_test_app().await;
    let client = Client::new();
    let username = String::from("josh");
    let password = String::from("freedman");
    // First, create a user
    let create_user_response = client
        .post(format!("http://{}/users", address))
        .json(&UserInput::new(
            username.clone(),
            password.clone(),
            password.clone(),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        create_user_response.status().as_u16(),
        200,
        "Response body as raw text: {:?}",
        create_user_response.text().await.unwrap()
    );
    // Then, validate that the user exists by logging in
    let login_response = client
        .post(format!("http://{}/login", address))
        .json(&Login::new(username.clone(), password.clone()))
        .send()
        .await
        .unwrap();
    assert_eq!(
        login_response.status().as_u16(),
        200,
        "Response body as raw text: {:?}",
        login_response.text().await.unwrap()
    );
}

#[tokio::test]
async fn create_user_email_already_exists() {
    todo!("Implement this test!")
}

#[tokio::test]
async fn create_user_password_does_not_match_requirements() {
    todo!("Implement this test!")
}
