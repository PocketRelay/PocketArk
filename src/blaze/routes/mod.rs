use super::models::EmptyData;
use super::router::Router;
use super::{components, session::Session};

mod auth;
mod util;

pub fn router() -> Router<Session> {
    let mut router = Router::new();

    router.route(
        (components::util::COMPONENT, components::util::PRE_AUTH),
        util::pre_auth,
    );
    router.route(
        (components::util::COMPONENT, components::util::POST_AUTH),
        util::post_auth,
    );
    router.route(
        (components::util::COMPONENT, components::util::PING),
        util::ping,
    );
    router.route(
        (
            components::authentication::COMPONENT,
            components::authentication::AUTHENTICATE,
        ),
        auth::auth,
    );
    router.route(
        (
            components::util::COMPONENT,
            components::util::FETCH_CLIENT_CONFIG,
        ),
        util::fetch_client_config,
    );
    router.route((0, 0), keep_alive);

    router
}

async fn keep_alive(_state: &mut Session, req: EmptyData) -> EmptyData {
    EmptyData
}

#[test]
fn test() {
    let s = "\x17CON".as_bytes();
    println!("{}", s[0])
}
