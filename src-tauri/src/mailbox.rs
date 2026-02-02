use futures::FutureExt;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;

pub struct LocalMailboxState {
    tx: tokio::sync::oneshot::Sender<()>,
    server: tokio::task::JoinHandle<()>,
}

type LocalMailbox = Mutex<Option<LocalMailboxState>>;

pub fn start_local_mailbox<R: Runtime>(
    handle: &AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    tauri::async_runtime::block_on(async move {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let rx = rx.map(|f| f.expect("failed to listen for event"));
        let path = handle.path().local_data_dir()?.join("local-mailbox.redb");
        let addr = format!(
            "0.0.0.0:{}",
            std::env::var("LOCAL_MAILBOX_PORT").unwrap_or_else(|_| "3411".to_string())
        );
        let server = tokio::spawn(async move {
            match mailbox_server::spawn_server(path, addr, rx).await {
                Ok(_) => (),
                Err(e) => log::error!("Failed to start local mailbox: {e:?}"),
            }
        });
        let state: LocalMailbox = Mutex::new(Some(LocalMailboxState { tx, server }));
        handle.manage(state);
        log::info!("Started local mailbox");
        Ok(())
    })
}

pub fn stop_local_mailbox<R: Runtime>(handle: &AppHandle<R>) {
    tauri::async_runtime::block_on(async move {
        let Some(state_mutex) = handle.try_state::<LocalMailbox>() else {
            log::warn!("Tried to stop local mailbox, but it was not running");
            return;
        };
        let Some(state) = state_mutex.lock().await.take() else {
            log::warn!("Tried to stop local mailbox, but it was not running");
            return;
        };
        log::info!("Sending stop signal to local mailbox...");
        let _ = state.tx.send(());
        state.server.await.unwrap();
        log::info!("Local mailbox stopped");
    })
}
