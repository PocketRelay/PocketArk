pub use sea_orm_migration::prelude::*;

mod m20230714_105755_create_users;
mod m20230714_105927_create_currency;
mod m20230714_105933_create_inventory_item;
mod m20230714_105940_create_seen_articles;
mod m20230714_105946_create_characters;
mod m20230714_112535_create_shared_data;
mod m20230720_145347_create_challenge_progress;
mod m20230731_123814_create_strike_teams;
mod m20231223_184934_create_strike_team_missions;
mod m20231223_185554_create_strike_team_mission_progress;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230714_105755_create_users::Migration),
            Box::new(m20230714_105927_create_currency::Migration),
            Box::new(m20230714_105933_create_inventory_item::Migration),
            Box::new(m20230714_105940_create_seen_articles::Migration),
            Box::new(m20230714_105946_create_characters::Migration),
            Box::new(m20230714_112535_create_shared_data::Migration),
            Box::new(m20230720_145347_create_challenge_progress::Migration),
            Box::new(m20230731_123814_create_strike_teams::Migration),
            Box::new(m20231223_184934_create_strike_team_missions::Migration),
            Box::new(m20231223_185554_create_strike_team_mission_progress::Migration),
        ]
    }
}
