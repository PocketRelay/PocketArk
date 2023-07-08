mod auth {
    /// POST /auth
    async fn authenticate() {}
}

mod configuration {

    /// GET /configuration
    async fn get_configuration() {}
}

mod mission {
    /// GET /mission/current
    async fn current_mission() {}

    /// GET /user/mission/:id
    async fn get_mission() {}

    /// POST /user/mission/:id/start
    async fn start_mission() {}

    /// POST /user/mission/:id/finish
    async fn finish_mission() {}
}

mod strike_teams {
    /// GET /striketeams
    async fn get() {}

    /// GET /striketeams/successRate
    async fn get_success_rate() {}
}

mod character {
    /// GET /characters
    async fn get_characters() {}

    /// GET /character/:id
    async fn get_character() {}

    /// POST /character/:id/active
    async fn set_active() {}

    /// GET /character/:id/equipment
    async fn get_character_equip() {}

    /// pUT /character/:id/equipment
    async fn update_character_equip() {}

    /// GET /character/:id/equipment/history
    async fn get_character_equip_history() {}

    /// PUT /character/:id/skillTrees
    async fn update_skill_tree() {}

    /// GET /character/classes
    async fn get_classes() {}

    /// GET /character/levelTables
    async fn get_level_tables() {}
}

mod challenge {

    /// GET /challenges
    async fn get_challenges() {}

    /// GET /challenges/user
    async fn get_user_challenges() {}
}

mod store {

    /// GET /store/catalogs
    async fn get_catalogs() {}

    /// POST /store/article
    async fn obtain_article() {}

    /// POST /store/unclaimed/claimAll
    async fn claim_unclaimed() {}

    /// GET /user/currencies
    async fn get_currencies() {}
}

mod inventory {

    /// GET /inventory
    async fn get_inventory() {}

    /// GET /inventory/definitions
    async fn get_definitions() {}

    /// PUT /inventory/seen
    async fn update_inventory_seen() {}

    /// POST /inventory/consume
    async fn consume_inventory() {}
}

mod leaderboard {
    /// GET /leaderboards
    async fn get_leaderboards() {}

    /// GET /leaderboards/:id
    async fn get_leaderboard() {}
}

mod presence {
    /// PUT /presence/session
    async fn update_session() {}
}

mod telemetry {
    /// POST /pinEvents
    async fn pin_events() {}
}

mod user_match {
    /// GET /user/match/badges
    async fn get_badges() {}

    /// GET /user/match/modifiers
    async fn get_modifiers() {}
}

mod activity {
    /// POST /activity
    async fn create_report() {}

    /// GET /activity/metadata
    async fn get_metadata() {}
}
