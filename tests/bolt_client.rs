use rust_bolt::Client;

#[tokio::test]
async fn handshake() {
    let mut client = Client::new("127.0.0.1".parse().unwrap(), 7687)
        .await
        .unwrap();
    assert_eq!(client.handshake().await.unwrap(), 1);
}
