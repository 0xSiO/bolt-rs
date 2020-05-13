use std::env;
use std::iter::FromIterator;

use criterion::*;
use tokio::runtime::Runtime;

use bolt_client::*;

async fn get_initialized_client() -> Result<Client, Box<dyn std::error::Error>> {
    let mut client = Client::new(
        env::var("BOLT_TEST_ADDR").unwrap(),
        env::var("BOLT_TEST_DOMAIN").ok(),
    )
    .await?;
    client.handshake(&[3, 0, 0, 0]).await?;
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
        let mut client = runtime.block_on(get_initialized_client()).unwrap();
        b.iter(|| {
            runtime.block_on(async {
                client
                    .run_with_metadata("RETURN 1 as num;", None, None)
                    .await
                    .unwrap();
                client.pull_all().await.unwrap();
            });
        });
    });
}

fn complex_query_bench(c: &mut Criterion) {
    let mut runtime = Runtime::new().unwrap();

    c.bench_function("complex query", |b| {
        let mut client = runtime.block_on(get_initialized_client()).unwrap();
        // Set up
        runtime.block_on(async {
            client.run_with_metadata(
                "CREATE (:Client {name: 'bolt-client', starting: datetime('2019-12-19T16:08:04-08:00'), test: 'bench-node-rel'})-[:WRITTEN_IN]->(:Language {name: 'Rust', test: 'bench-node-rel'});",
                None, None).await.unwrap();
            client.pull_all().await.unwrap();
        });

        b.iter(|| {
            runtime.block_on(async {
                client.run_with_metadata("MATCH (c {test: 'bench-node-rel'})-[r:WRITTEN_IN]->(l) RETURN c, r, l;", None, None).await.unwrap();
                let (_response, _records) = client.pull_all().await.unwrap();
            });
        });

        // Clean up
        runtime.block_on(async {
            client.run_with_metadata("MATCH (n {test: 'bench-node-rel'}) DETACH DELETE n;", None, None).await.unwrap();
            client.pull_all().await.unwrap();
        });
    });
}

criterion_group!(
    basic_benches,
    initialize_client_bench,
    simple_query_bench,
    complex_query_bench,
);
criterion_main!(basic_benches);
