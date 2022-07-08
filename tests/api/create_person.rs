use crate::helpers::spawn_test_app;
use reqwest::Client;
use testcontainers::clients::Cli;
use testcontainers_test::domain::{Person, PersonInput};
use tokio;

#[tokio::test]
async fn create_person() {
    let docker = Cli::default();
    let test_app = spawn_test_app(&docker);
    let person_input = PersonInput::new("Josh");
    let client = Client::new();
    let post_response = client
        .post(format!("http://{}/people", test_app.address))
        .json(&person_input)
        .send()
        .await
        .unwrap();
    assert_eq!(
        post_response.status().as_u16(),
        200,
        "Unsuccessful response: {:?}",
        post_response.text().await.unwrap()
    );
    assert_eq!(
        post_response.headers().get("content-type").unwrap(),
        "application/json",
        "Content-Type is be application/json"
    );
    let newly_created_person = post_response.json::<Person>().await.unwrap();
    assert_eq!(newly_created_person.name(), "Josh");

    let get_response = client
        .get(format!(
            "http://{}/people/{}",
            test_app.address,
            newly_created_person.id()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get_response.status().as_u16(),
        200,
        "Unsuccessful response: {:?}",
        get_response.text().await.unwrap()
    );
    assert_eq!(
        get_response.headers().get("content-type").unwrap(),
        "application/json",
        "Content-Type is not application/json"
    );
    let retrieved_person = get_response.json::<Person>().await.unwrap();
    assert_eq!(newly_created_person, retrieved_person);
}

/// This test mostly exists to assert that each connection
#[tokio::test]
async fn get_person_expect_empty() {
    let docker = Cli::default();
    let test_app = spawn_test_app(&docker);
    let client = Client::new();
    let get_response = client
        .get(format!("http://{}/people/1", test_app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get_response.status().as_u16(),
        404,
        "Didn't get a 404, instead got response: {:?}",
        get_response.text().await.unwrap()
    );
}
