pub use sea_orm_migration::prelude::*;

mod m20230714_105755_create_users;
mod m20230714_105927_create_currency;
mod m20230714_105933_create_inventory_item;
mod m20230714_105940_create_seen_articles;
mod m20230714_105946_create_charcters;

pub struct Migrator;
m20230714_105755_create_users
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230714_105755_create_players::Migration),
            Box::new(m20230714_105927_create_currency::Migration),
            Box::new(m20230714_105933_create_inventory_item::Migration),
            Box::new(m20230714_105940_create_seen_articles::Migration),
            Box::new(m20230714_105946_create_charcters::Migration),
        ]
    }
}
