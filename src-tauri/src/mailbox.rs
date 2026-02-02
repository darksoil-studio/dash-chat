use futures::FutureExt;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;

pub struct LocalMailboxState {
    stop_signal: tokio::sync::oneshot::Sender<()>,
    server: tokio::task::JoinHandle<()>,
}

pub(crate) type LocalMailboxMutex = Mutex<Option<LocalMailboxState>>;

pub fn start_local_mailbox<R: Runtime>(
    handle: &AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    tauri::async_runtime::block_on(async move {
        let state_mutex = handle.state::<LocalMailboxMutex>();
        let mut state = state_mutex.lock().await;
        if state.is_some() {
            return Ok(());
        }

        let (stop_signal_tx, stop_signal_rx) = tokio::sync::oneshot::channel();
        let stop_signal_rx = stop_signal_rx.map(|f| f.expect("failed to listen for event"));
        let path = handle.path().local_data_dir()?.join("local-mailbox.redb");
        let addr = format!(
            "0.0.0.0:{}",
            std::env::var("LOCAL_MAILBOX_PORT").unwrap_or_else(|_| "3411".to_string())
        );
        let server = tokio::spawn(async move {
            match mailbox_server::spawn_server(path, addr, stop_signal_rx).await {
                Ok(_) => (),
                Err(e) => log::error!("Failed to start local mailbox: {e:?}"),
            }
        });
        log::info!("Started local mailbox");
        if state
            .replace(LocalMailboxState {
                stop_signal: stop_signal_tx,
                server,
            })
            .is_some()
        {
            unreachable!("Replaced existing mailbox state with new state, this should not happen.");
        }
        Ok(())
    })
}

pub fn stop_local_mailbox<R: Runtime>(handle: &AppHandle<R>) {
    tauri::async_runtime::block_on(async move {
        let state_mutex = handle.state::<LocalMailboxMutex>();
        let mut state = state_mutex.lock().await;
        let Some(state) = state.take() else {
            log::warn!("Tried to stop local mailbox, but it was not running");
            return;
        };
        log::info!("Sending stop signal to local mailbox...");
        let _ = state.stop_signal.send(());
        state.server.await.unwrap();
        log::info!("Local mailbox stopped");
    })
}
