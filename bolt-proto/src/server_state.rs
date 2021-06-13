#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
