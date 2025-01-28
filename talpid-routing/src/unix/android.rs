use std::sync::Mutex;

use crate::imp::RouteManagerCommand;
use futures::{channel::mpsc::{self, UnboundedReceiver, UnboundedSender}, stream::StreamExt};
use ipnetwork::IpNetwork;
use talpid_types::android::AndroidContext;
use jnix::{jni::{objects::JObject, sys::jboolean, JNIEnv}, JnixEnv};

/// Stub error type for routing errors on Android.
/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send shutdown result")]
    SendError,

    #[error("Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[source] jnix::jni::errors::Error),

    #[error("Failed to call Java method TalpidVpnService.{0}")]
    //CallMethod(&'static str, #[source] jnix::jni::errors::Error),
    CallMethod(&'static str),

    #[error("Failed to create Java VM handle clone")]
    CloneJavaVm(#[source] jnix::jni::errors::Error),

    #[error("Failed to find TalpidVpnService.{0} method")]
    FindMethod(&'static str, #[source] jnix::jni::errors::Error),

    #[error("Received an invalid result from TalpidVpnService.{0}: {1}")]
    InvalidMethodResult(&'static str, String),

    #[error("Routes timed out")]
    RoutesTimedOut,

    #[error("Profile for VPN has not been setup")]
    NotPrepared,

    #[error("Another legacy VPN profile is used as always on")]
    OtherLegacyAlwaysOnVpn,
}

/// TODO: Document mee
static ROUTE_UPDATES_TX: Mutex<Option<UnboundedSender<RoutesUpdate>>> = Mutex::new(None);

pub enum RoutesUpdate {
    NewRoutes(Routes),
}

// TODO: This is le actor state
/// Stub route manager for Android
pub struct RouteManagerImpl {
    android: AndroidContext,
    routes_udates: UnboundedReceiver<RoutesUpdate>,
    listeners: Vec<UnboundedSender<RoutesUpdate>>,
}

pub enum RouteResult {
    CorrectRoutes,
    IncorrectRoutes,
}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new(android: AndroidContext) -> Result<Self, Error> {
        // Create a channel between the kotlin client and route manager
        let (tx, rx) = futures::channel::mpsc::unbounded();
        // TODO: What id `ROUTE_UPDATES_TX` has already been initialized?
        *ROUTE_UPDATES_TX.lock().unwrap() = Some(tx);
        Ok(RouteManagerImpl {
            android,
            routes_udates: rx,
            listeners: Default::default(),
        })
    }

    pub(crate) async fn run(
        self,
        manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<(), Error> {
        let mut manage_rx = manage_rx.fuse();
        while let Some(command) = manage_rx.next().await {
            match command {
                RouteManagerCommand::NewChangeListener(tx ) => {
                    // register a listener for new route updates
                    let _ = result_tx.send(self.listen());
                }
                RouteManagerCommand::Shutdown(tx) => {
                    tx.send(()).map_err(|()| Error::SendError)?; // TODO: Surely we can do better than this
                    break;
                }
                RouteManagerCommand::AddRoutes(_routes, tx) => {
                    tx.send(Ok(())).map_err(|_x| Error::SendError)?;
                }
                RouteManagerCommand::ClearRoutes => (),
            }
        }
        Ok(())
    }

    fn notify_change_listeners(&mut self, message: RoutesUpdate) {
        self.listeners
            .retain(|listener| listener.unbounded_send(message.clone()).is_ok());
    }

    fn listen(&mut self) -> UnboundedReceiver<CallbackMessage> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.listeners.push(tx);
        rx
    }
}

// TODO: name
#[derive(FromJava)]
struct Routes {
    routes: Vec<IpNetwork>,
}

/// Entry point for Android Java code to notify the connectivity status.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_notifyRoutesChanged(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    routes: JObject<'_>, // TODO: Actually get the routes
) {
    let routes = Routes::from_java(routes);
    let Some(tx) = &*ROUTE_UPDATES_TX.lock().unwrap() else {
        // No sender has been registered
        log::trace!("Received eroutes notification wíth no channel");
        return;
    };

    if tx
        .unbounded_send(RoutesUpdate::NewRoutes(routes))
        .is_err()
    {
        log::warn!("Failed to send offline change event");
    }
}