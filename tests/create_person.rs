use reqwest::Client;
use std::net::TcpListener;
use testcontainers_test::{run, Person, PersonInput};
use tokio;

#[tokio::test]
async fn create_person() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let _ = tokio::spawn(async move {
        run(listener).unwrap().await.unwrap();
    });
    let person_input = PersonInput::new("Josh");
    let client = Client::new();
    let post_response = client
        .post(format!("http://{}/people", address))
        .json(&person_input)
        .send()
        .await
        .unwrap();
    assert_eq!(
        post_response.status().as_u16(),
        200,
        "Response was not successful"
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
            address,
            newly_created_person.id()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get_response.status().as_u16(),
        200,
        "Response was not successful"
    );
    assert_eq!(
        get_response.headers().get("content-type").unwrap(),
        "application/json",
        "Content-Type is be application/json"
    );
    let retrieved_person = get_response.json::<Person>().await.unwrap();
    assert_eq!(newly_created_person, retrieved_person);
}
