use failure::Error;

use bolt_proto::Client;

async fn new_client() -> Result<Client, Error> {
    Client::new("127.0.0.1".parse().unwrap(), 7687).await
}

#[tokio::test]
async fn handshake() {
    let mut client = new_client().await.unwrap();
    assert_eq!(client.handshake().await.unwrap(), 1);
}

#[tokio::test]
async fn init() {
    let mut client = new_client().await.unwrap();
    assert_eq!(client.handshake().await.unwrap(), 1);
    println!("{:?}", client.init().await);
}
