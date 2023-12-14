use crate::services::Services;

/// The server version extracted from the Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Global state that is shared throughout the application this
/// will be unset until the value is initialized then it will be
/// set
pub struct App {
    /// Global session router
    pub services: Services,
}

/// Static global state value
static mut GLOBAL_STATE: Option<App> = None;

impl App {
    pub fn init() {
        // Initialize session router
        let services = Services::init();

        unsafe {
            GLOBAL_STATE = Some(App { services });
        }
    }

    pub fn services() -> &'static Services {
        match unsafe { &GLOBAL_STATE } {
            Some(value) => &value.services,
            None => panic!("Global state not initialized"),
        }
    }
}
