#[derive(Debug)]
pub enum ServerState {
    Disconnected,
    Connected,
    Defunct,
    Ready,
    Streaming,
    TxReady,
    TxStreaming,
    Failed,
    Interrupted,
}
