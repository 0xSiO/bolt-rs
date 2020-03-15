use std::collections::HashMap;
use std::env;
use std::iter::FromIterator;

use criterion::*;
use tokio::runtime::Runtime;

use bolt_client::error::Result;
use bolt_client::*;

async fn get_initialized_client() -> Result<Client> {
    let mut client = Client::new(
        env::var("BOLT_TEST_ADDR").unwrap(),
        env::var("BOLT_TEST_DOMAIN").ok().as_deref(),
    )
    .await?;
    client.handshake(&[1, 0, 0, 0]).await?; // TODO: Update this to use higher versions when implemented
    client
        .init(
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (
                    String::from("principal"),
                    env::var("BOLT_TEST_USERNAME").unwrap(),
                ),
                (
                    String::from("credentials"),
                    env::var("BOLT_TEST_PASSWORD").unwrap(),
                ),
            ]),
        )
        .await?;
    Ok(client)
}

fn initialize_client_bench(c: &mut Criterion) {
    let mut runtime = Runtime::new().unwrap();

    c.bench_function("init client", |b| {
        b.iter(|| {
            runtime.block_on(async { get_initialized_client().await.unwrap() });
        })
    });
}

fn simple_query_bench(c: &mut Criterion) {
    let mut runtime = Runtime::new().unwrap();

    c.bench_function("simple query", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let mut client = get_initialized_client().await.unwrap();
                client
                    .run("RETURN 1 as num;".to_string(), None)
                    .await
                    .unwrap();
                client.pull_all().await.unwrap();
            });
        })
    });
}

criterion_group!(benches, initialize_client_bench, simple_query_bench,);
criterion_main!(benches);
