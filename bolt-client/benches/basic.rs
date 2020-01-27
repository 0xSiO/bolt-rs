use std::collections::HashMap;
use std::iter::FromIterator;

use criterion::*;
use failure::Fallible;
use tokio::runtime::Runtime;

use bolt_client::*;

async fn get_initialized_client() -> Fallible<Client> {
    let mut client: Client = Client::new_tcp("127.0.0.1:7687").await?;
    client
        .init(
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (String::from("principal"), String::from("neo4j")),
                (String::from("credentials"), String::from("test")),
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
