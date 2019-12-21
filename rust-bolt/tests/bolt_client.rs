use std::error::Error;

use rust_bolt::Client;

async fn new_client() -> Result<Client, Box<dyn Error>> {
    Client::new("127.0.0.1".parse().unwrap(), 7687).await
}

#[tokio::test]
async fn handshake() {
    let mut client = new_client().await.unwrap();
    assert_eq!(client.handshake().await.unwrap(), 1);
}
