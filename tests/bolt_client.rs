use rust_bolt::BoltClient;

#[tokio::test]
async fn handshake() {
    let mut client = BoltClient::new("127.0.0.1".parse().unwrap(), 7687).await.unwrap();
    assert_eq!(client.handshake().await.unwrap(), 1);
}
