use std::path::PathBuf;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::stream;
use iced::Subscription;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::message::Message;

enum WatchMsg {
    Path(PathBuf),
    Failed(String),
}

fn watch_builder(data: &(u64, PathBuf)) -> impl Stream<Item = Message> {
    let watch_root = data.1.clone();
    stream::channel(256, async move |mut output: mpsc::Sender<Message>| {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WatchMsg>();

        let root = watch_root.clone();
        // Clone tx before moving into the closure so we can use it after the match
        let tx_for_watcher = tx.clone();
        std::thread::spawn(move || {
            let mut watcher = match RecommendedWatcher::new(
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(ev) = res {
                        if matches!(ev.kind, EventKind::Access(_)) {
                            return;
                        }
                        for p in ev.paths {
                            let _ = tx_for_watcher.send(WatchMsg::Path(p));
                        }
                    }
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    let _ = tx.send(WatchMsg::Failed(e.to_string()));
                    return;
                }
            };

            if let Err(e) = watcher.watch(&root, RecursiveMode::Recursive) {
                let _ = tx.send(WatchMsg::Failed(e.to_string()));
                return;
            }

            loop {
                std::thread::park();
            }
        });

        while let Some(msg) = rx.recv().await {
            match msg {
                WatchMsg::Failed(e) => {
                    let _ = output.send(Message::WatcherInitFailed(e)).await;
                }
                WatchMsg::Path(p) => {
                    let _ = output.send(Message::FsChange(p)).await;
                }
            }
        }
    })
}

/// Subscribe to filesystem changes under `watch_root`.
pub fn watch_subscription(watcher_id: u64, watch_root: PathBuf) -> Subscription<Message> {
    Subscription::run_with((watcher_id, watch_root), watch_builder)
}
