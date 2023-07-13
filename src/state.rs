use sea_orm::DatabaseConnection;
use tokio::join;

use crate::{
    blaze::{self, pk::router::Router, session::SessionLink},
    services::Services,
};

/// The server version extracted from the Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Global state that is shared throughout the application this
/// will be unset until the value is initialized then it will be
/// set
pub struct App {
    /// Global session router
    pub router: Router<SessionLink>,
    pub database: DatabaseConnection,
    pub services: Services,
}

/// Static global state value
static mut GLOBAL_STATE: Option<App> = None;

impl App {
    pub async fn init() {
        // Initialize session router
        let router = blaze::routes::router();
        let services = Services::init().await;
        let database = crate::database::init().await;

        unsafe {
            GLOBAL_STATE = Some(App {
                router,
                services,
                database,
            });
        }
    }

    /// Obtains a static reference to the session router
    pub fn router() -> &'static Router<SessionLink> {
        match unsafe { &GLOBAL_STATE } {
            Some(value) => &value.router,
            None => panic!("Global state not initialized"),
        }
    }

    pub fn services() -> &'static Services {
        match unsafe { &GLOBAL_STATE } {
            Some(value) => &value.services,
            None => panic!("Global state not initialized"),
        }
    }
    pub fn database() -> &'static DatabaseConnection {
        match unsafe { &GLOBAL_STATE } {
            Some(value) => &value.database,
            None => panic!("Global state not initialized"),
        }
    }
}
