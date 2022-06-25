use postgres::{Client, NoTls};
use testcontainers::{clients::Cli, core::WaitFor, images};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

fn main() {
    let db = "postgres-db-test";
    let user = "postgres-user-test";
    let password = "postgres-password-test";
    let port = 5432;

    let docker = Cli::default();
    let postgres = images::generic::GenericImage::new("postgres", "14.4-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", db)
        .with_env_var("POSTGRES_USER", user)
        .with_env_var("POSTGRES_PASSWORD", password)
        .with_exposed_port(port);
    let node = docker.run(postgres);
    let mut client = Client::connect(
        format!(
            "host=localhost port={} user={} password={} dbname={}",
            node.get_host_port_ipv4(port),
            user,
            password,
            db
        )
        .as_str(),
        NoTls,
    )
    .unwrap();

    embedded::migrations::runner().run(&mut client).unwrap();

    let name = "Ferris";
    let data = None::<&[u8]>;
    client
        .execute(
            "INSERT INTO person (name, data) VALUES ($1, $2)",
            &[&name, &data],
        )
        .unwrap();

    for row in client
        .query("SELECT id, name, data FROM person", &[])
        .unwrap()
    {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let data: Option<&[u8]> = row.get(2);

        println!("found person: {} {} {:?}", id, name, data);
    }
}
