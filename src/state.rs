use tokio::join;

use crate::blaze::{self, pk::router::Router, session::SessionLink};

/// The server version extracted from the Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Global state that is shared throughout the application this
/// will be unset until the value is initialized then it will be
/// set
pub struct App {
    /// Global session router
    pub router: Router<SessionLink>,
}

/// Static global state value
static mut GLOBAL_STATE: Option<App> = None;

impl App {
    pub async fn init() {
        // Initialize session router
        let router = blaze::routes::router();

        unsafe {
            GLOBAL_STATE = Some(App { router });
        }
    }

    /// Obtains a static reference to the session router
    pub fn router() -> &'static Router<SessionLink> {
        match unsafe { &GLOBAL_STATE } {
            Some(value) => &value.router,
            None => panic!("Global state not initialized"),
        }
    }
}
