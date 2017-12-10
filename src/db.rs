use std::path::Path;
use failure::Error;
use rusqlite::{Connection, OpenFlags, Row};
use rusqlite::types::ToSql;

pub struct DB {
    conn: Connection
}

pub trait TableRow where Self: Sized {
    const TABLE_NAME: &'static str;
    const CREATE_TABLE: &'static str;
    const INSERT: &'static str;
    fn from_row(r: &Row) -> Result<Self, Error>;
    fn to_insert_params(self: &Self) -> Vec<&ToSql>;
}

impl DB {
    pub fn open_read_write<P: AsRef<Path>>(path: P) -> Result<DB, Error> {
        let conn = Connection::open(path)?;
        conn.execute("PRAGMA foreign_keys = ON", &[])?;
        Ok(DB { conn })
    }

    pub fn open_read_only<P: AsRef<Path>>(path: P) -> Result<DB, Error> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(DB { conn })
    }

    pub fn make_table<T: TableRow>(self: &mut DB) -> Result<(), Error> {
        self.conn.execute(T::CREATE_TABLE, &[])?;
        Ok(())
    }

    pub fn select_one<T: TableRow>(self: &mut DB) -> Result<T, Error> {
        let query_str = format!("SELECT * FROM {}", T::TABLE_NAME);
        self.conn.query_row(&query_str, &[], T::from_row)?
    }

    pub fn select_one_where<T: TableRow>(self: &mut DB, query: &str, params: &[&ToSql]) -> Result<T, Error> {
        let query_str = format!("SELECT * FROM {} WHERE {}", T::TABLE_NAME, query);
        self.conn.query_row(&query_str, params, T::from_row)?
    }

    pub fn select_all<T: TableRow>(self: &mut DB) -> Result<Vec<T>, Error> {
        let query_str = format!("SELECT * FROM {}", T::TABLE_NAME);
        let mut stmt = self.conn.prepare(&query_str)?;
        let rows = stmt.query_and_then(&[], T::from_row)?;
        let mut items = Vec::new();
        for result in rows {
            let item = result?;
            items.push(item);
        }
        Ok(items)
    }

    pub fn select_all_where<T: TableRow>(self: &mut DB, query: &str, params: &[&ToSql]) -> Result<Vec<T>, Error> {
        let query_str = format!("SELECT * FROM {} WHERE {}", T::TABLE_NAME, query);
        let mut stmt = self.conn.prepare(&query_str)?;
        let rows = stmt.query_and_then(params, T::from_row)?;
        let mut items = Vec::new();
        for result in rows {
            let item = result?;
            items.push(item);
        }
        Ok(items)
    }

    pub fn insert_row<T: TableRow>(self: &mut DB, t: &T) -> Result<(), Error> {
        let mut stmt = self.conn.prepare(T::INSERT)?;
        let params = t.to_insert_params();
        stmt.insert(&params)?;
        Ok(())
    }
}

// vi: ts=8 sts=4 et
