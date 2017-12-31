use std::path::Path;
use failure::{Error, err_msg};
use rusqlite::{Connection, OpenFlags, Row};
use rusqlite::types::ToSql;

pub struct DB {
    conn: Connection
}

pub trait Table {
    type TableRow: Sized;
    const TABLE_NAME: &'static str;
    const CREATE_TABLE: &'static str;
    const INSERT: &'static str;
    fn from_row(r: &Row) -> Result<Self::TableRow, Error>;
    fn do_insert<F>(r: &Self::TableRow, insert: F) -> Result<(), Error>
        where F: FnOnce(&[&ToSql]) -> Result<(), Error>;
}

pub trait TableUpdate<R> where Self: Table {
    const UPDATE: &'static str;
    fn do_update<F>(r: &R, update: F) -> Result<(), Error>
        where F: FnOnce(&[&ToSql]) -> Result<(), Error>;
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

    pub fn create_table<T: Table>(self: &mut DB) -> Result<(), Error> {
        self.conn.execute(T::CREATE_TABLE, &[])?;
        Ok(())
    }

    pub fn select_one<T: Table>(self: &mut DB) -> Result<T::TableRow, Error> {
        let query_str = format!("SELECT * FROM {}", T::TABLE_NAME);
        self.conn.query_row(&query_str, &[], T::from_row)?
    }

    pub fn select_one_where<T: Table>(self: &mut DB, query: &str, params: &[&ToSql]) -> Result<T::TableRow, Error> {
        let query_str = format!("SELECT * FROM {} WHERE {}", T::TABLE_NAME, query);
        self.conn.query_row(&query_str, params, T::from_row)?
    }

    pub fn select_all<T: Table>(self: &mut DB) -> Result<Vec<T::TableRow>, Error> {
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

    pub fn select_all_where<T: Table>(self: &mut DB, query: &str, params: &[&ToSql]) -> Result<Vec<T::TableRow>, Error> {
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

    pub fn insert_row<T: Table>(self: &mut DB, r: &T::TableRow) -> Result<(), Error> {
        let query_str = format!("INSERT INTO {} {}", T::TABLE_NAME, T::INSERT);
        let mut stmt = self.conn.prepare(&query_str)?;
        T::do_insert(r, |params| {
            stmt.insert(&params)?;
            Ok(())
        })
    }

    pub fn update_row<T, R>(self: &mut DB, r: &R) -> Result<(), Error>
    where T: TableUpdate<R> {
        let query_str = format!("UPDATE {} SET {}", T::TABLE_NAME, T::UPDATE);
        let mut stmt = self.conn.prepare(&query_str)?;
        T::do_update(r, |params| {
            let count = stmt.execute(params)?;
            if count == 1 {
                Ok(())
            } else if count > 1 {
                Err(err_msg("multiple rows updated"))
            } else {
                Err(err_msg("no rows updated"))
            }
        })
    }

    pub fn update_many<T, R>(self: &mut DB, r: &R) -> Result<(), Error>
    where T: TableUpdate<R> {
        let query_str = format!("UPDATE {} SET {}", T::TABLE_NAME, T::UPDATE);
        let mut stmt = self.conn.prepare(&query_str)?;
        T::do_update(r, |params| {
            let count = stmt.execute(params)?;
            if count > 0 {
                Ok(())
            } else {
                Err(err_msg("no rows updated"))
            }
        })
    }
}

// vi: ts=8 sts=4 et
