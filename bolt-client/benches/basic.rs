use std::env;
use std::iter::FromIterator;

use criterion::*;
use tokio::runtime::Runtime;

use bolt_client::*;

async fn get_initialized_client() -> Result<Client, Box<dyn std::error::Error>> {
    let mut client = Client::new(
        env::var("BOLT_TEST_ADDR").unwrap(),
        env::var("BOLT_TEST_DOMAIN").ok().as_deref(),
    )
    .await?;
    client.handshake(&[3, 2, 1, 0]).await?; // TODO: Should we benchmark multiple client versions?
    client
        .hello(Some(Metadata::from_iter(vec![
            ("user_agent".to_string(), "bolt-client/X.Y.Z".to_string()),
            ("scheme".to_string(), "basic".to_string()),
            ("principal".to_string(), env::var("BOLT_TEST_USERNAME")?),
            ("credentials".to_string(), env::var("BOLT_TEST_PASSWORD")?),
        ])))
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
                    .run_with_metadata("RETURN 1 as num;", None, None)
                    .await
                    .unwrap();
                client.pull_all().await.unwrap();
            });
        })
    });
}

criterion_group!(benches, initialize_client_bench, simple_query_bench,);
criterion_main!(benches);
