use serde::Serialize;
use sqlx::{Database, Encode, Type};

/// Extension helper trait that provides a easy way to bind many
/// parameters without using a temporary variable and reassignment
/// for collections of data stored as an iterable type
pub trait SqlxBindExt<'q, DB: Database>: Sized {
    fn bind_all<T, I>(self, values: I) -> Self
    where
        T: 'q + Encode<'q, DB> + Type<DB>,
        I: IntoIterator<Item = T>;

    fn bind_json<'t, T: Serialize>(self, value: T) -> Result<Self, sqlx::Error>
    where
        String: 'q + Encode<'q, DB> + Type<DB>;
}

impl<'q, DB, O> SqlxBindExt<'q, DB> for sqlx::query::QueryAs<'q, DB, O, DB::Arguments>
where
    DB: Database,
{
    fn bind_all<T, I>(mut self, values: I) -> Self
    where
        T: 'q + Encode<'q, DB> + Type<DB>,
        I: IntoIterator<Item = T>,
    {
        for value in values {
            self = self.bind(value);
        }
        self
    }

    fn bind_json<'t, T: Serialize>(self, value: T) -> Result<Self, sqlx::Error>
    where
        String: 'q + Encode<'q, DB> + Type<DB>,
    {
        let value = serde_json::to_string(&value).map_err(|err| sqlx::Error::Encode(err.into()))?;
        Ok(self.bind(value))
    }
}

impl<'q, DB> SqlxBindExt<'q, DB> for sqlx::query::Query<'q, DB, <DB as Database>::Arguments>
where
    DB: Database,
{
    fn bind_all<T, I>(mut self, values: I) -> Self
    where
        T: 'q + Encode<'q, DB> + Type<DB>,
        I: IntoIterator<Item = T>,
    {
        for value in values {
            self = self.bind(value);
        }
        self
    }

    fn bind_json<'t, T: Serialize>(self, value: T) -> Result<Self, sqlx::Error>
    where
        String: 'q + Encode<'q, DB> + Type<DB>,
    {
        let value = serde_json::to_string(&value).map_err(|err| sqlx::Error::Encode(err.into()))?;
        Ok(self.bind(value))
    }
}
