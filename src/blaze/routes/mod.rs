use std::future::ready;

use super::{components, session::SessionLink};
use crate::blaze::pk::router::Router;

mod auth;
mod game_manager;
mod user_sessions;
mod util;

pub fn router() -> Router<SessionLink> {
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
            components::authentication::COMPONENT,
            components::authentication::LIST_ENTITLEMENTS_2,
        ),
        auth::list_entitlements_2,
    );
    router.route(
        (
            components::util::COMPONENT,
            components::util::FETCH_CLIENT_CONFIG,
        ),
        util::fetch_client_config,
    );

    router.route(
        (
            components::user_sessions::COMPONENT,
            components::user_sessions::UPDATE_NETWORK_INFO,
        ),
        user_sessions::update_network_info,
    );
    router.route(
        (
            components::user_sessions::COMPONENT,
            components::user_sessions::UPDATE_HARDWARE_FLAGS,
        ),
        user_sessions::update_hardware_flags,
    );

    router.route(
        (
            components::game_manager::COMPONENT,
            components::game_manager::START_MATCHMAKING,
        ),
        game_manager::create_game,
    );
    router.route(
        (
            components::game_manager::COMPONENT,
            components::game_manager::UPDATE_PLAYER_ATTR,
        ),
        game_manager::update_player_attr,
    );
    router.route(
        (
            components::game_manager::COMPONENT,
            components::game_manager::UPDATE_GAME_STATE,
        ),
        game_manager::update_game_state,
    );
    router.route(
        (
            components::game_manager::COMPONENT,
            components::game_manager::UPDATE_GAME_ATTR,
        ),
        game_manager::update_game_attr,
    );
    router.route(
        (
            components::game_manager::COMPONENT,
            components::game_manager::REPLY_GAME,
        ),
        game_manager::replay_game,
    );

    router.route((0, 0), move |_: &'_ mut SessionLink| ready(()));

    router
}
