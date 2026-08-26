use crate::api::DEFAULT_SERVER_WS_HOST;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub enum OutboundMessage {
    ViewRequest {
        library: String,
        library_filter: Option<String>,
        sort: Option<String>,
        reverse: bool,
        filter_key: Option<String>,
        filter_val: Option<String>,
    },
    GroupRequest {
        library: String,
        library_filter: Option<String>,
        key: String,
    },
}

pub struct SyncEngine {
    inbound_rx: Receiver<Value>,
    outbound_tx: Sender<OutboundMessage>,
}

impl SyncEngine {
    #[must_use]
    pub fn start() -> Self {
        let (inbound_tx, inbound_rx) = std::sync::mpsc::channel();
        let (outbound_tx, outbound_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime");

            rt.block_on(sync_worker(inbound_tx, outbound_rx));
        });

        Self {
            inbound_rx,
            outbound_tx,
        }
    }

    #[must_use]
    pub fn poll_messages(&self) -> Vec<Value> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.inbound_rx.try_recv() {
            messages.push(msg);
        }
        messages
    }

    pub fn send(&self, msg: OutboundMessage) {
        let _ = self.outbound_tx.send(msg);
    }
}

async fn sync_worker(
    inbound_tx: Sender<Value>,
    outbound_rx: Receiver<OutboundMessage>,
) {
    loop {
        if let Ok((ws_stream, _)) = connect_async(DEFAULT_SERVER_WS_HOST).await {
            let (mut write, mut read) = ws_stream.split();

            loop {
                tokio::select! {
                    Some(msg) = read.next() => {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Ok(json_val) = serde_json::from_str::<Value>(&text) {
                                    let _ = inbound_tx.send(json_val);
                                }
                            }
                            Ok(Message::Close(_)) | Err(_) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    () = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                        while let Ok(outbound) = outbound_rx.try_recv() {
                            let json_str = match outbound {
                                OutboundMessage::ViewRequest {
                                    library,
                                    library_filter,
                                    sort,
                                    reverse,
                                    filter_key,
                                    filter_val,
                                } => {
                                    let filter_val_obj = match (filter_key, filter_val) {
                                        (Some(k), Some(v)) => serde_json::json!({ "key": k, "val": v }),
                                        _ => serde_json::json!({ "key": null, "val": null }),
                                    };
                                    serde_json::json!({
                                        "type": "VIEW_REQUEST",
                                        "library": library,
                                        "library_filter": library_filter,
                                        "sort": sort,
                                        "reverse": reverse,
                                        "filter": filter_val_obj
                                    }).to_string()
                                }
                                OutboundMessage::GroupRequest {
                                    library,
                                    library_filter,
                                    key,
                                } => {
                                    serde_json::json!({
                                        "type": "GROUP_REQUEST",
                                        "library": library,
                                        "library_filter": library_filter,
                                        "key": key
                                    }).to_string()
                                }
                            };
                            let _ = write.send(Message::Text(json_str.into())).await;
                        }
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
