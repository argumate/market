use std::path::Path;
use std::marker::PhantomData;
use failure::{Error, err_msg};
use rusqlite::{Connection, OpenFlags, Row};
use rusqlite::types::ToSql;

pub struct Select<'a, T> where T: Table {
    conn: &'a Connection,
    phantom: PhantomData<T>
}

pub struct Update<'a, T> where T: Table {
    conn: &'a Connection,
    phantom: PhantomData<T>
}

impl<'a, T> Select<'a, T> where T: Table {
    pub fn one(self: &Self) -> Result<T::TableRow, Error> {
        let query_str = format!("SELECT * FROM {}", T::TABLE_NAME);
        self.conn.query_row(&query_str, &[], T::from_row)?
    }

    pub fn one_where(self: &Self, query: &str, params: &[&ToSql]) -> Result<T::TableRow, Error> {
        let query_str = format!("SELECT * FROM {} WHERE {}", T::TABLE_NAME, query);
        self.conn.query_row(&query_str, params, T::from_row)?
    }

    pub fn all(self: &Self) -> Result<Vec<T::TableRow>, Error> {
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

    pub fn all_where(self: &Self, query: &str, params: &[&ToSql]) -> Result<Vec<T::TableRow>, Error> {
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
}

impl<'a, T> Update<'a, T> where T: Table {
    pub fn insert(self: &Self, query: &str, params: &[&ToSql]) -> Result<(), Error> {
        let query_str = format!("INSERT INTO {} {}", T::TABLE_NAME, query);
        let mut stmt = self.conn.prepare(&query_str)?;
        stmt.insert(&params)?;
        Ok(())
    }

    pub fn update_one(self: &Self, query: &str, params: &[&ToSql]) -> Result<(), Error> {
        let query_str = format!("UPDATE {} SET {}", T::TABLE_NAME, query);
        let mut stmt = self.conn.prepare(&query_str)?;
        let count = stmt.execute(params)?;
        if count == 1 {
            Ok(())
        } else if count > 1 {
            Err(err_msg("multiple rows updated"))
        } else {
            Err(err_msg("no rows updated"))
        }
    }

    pub fn update_many(self: &Self, query: &str, params: &[&ToSql]) -> Result<(), Error> {
        let query_str = format!("UPDATE {} SET {}", T::TABLE_NAME, query);
        let mut stmt = self.conn.prepare(&query_str)?;
        let count = stmt.execute(params)?;
        if count > 0 {
            Ok(())
        } else {
            Err(err_msg("no rows updated"))
        }
    }
}

pub trait Table where Self: Sized {
    type TableRow: Sized;
    const TABLE_NAME: &'static str;
    const CREATE_TABLE: &'static str;
    fn from_row(r: &Row) -> Result<Self::TableRow, Error>;
    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error>;
}

pub trait DB where Self: Sized {
    fn open_read_write<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
    fn open_read_only<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
    fn create_table<T: Table>(self: &Self) -> Result<(), Error>;
    fn select<'a, T: Table>(self: &'a Self) -> Select<'a, T>;
    fn insert<T: Table>(self: &Self, r: &T::TableRow) -> Result<(), Error>;
    fn update<'a, T: Table>(self: &'a Self) -> Update<'a, T>;
}

impl DB for Connection {
    fn open_read_write<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let conn = Connection::open(path)?;
        conn.execute("PRAGMA foreign_keys = ON", &[])?;
        Ok(conn)
    }

    fn open_read_only<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(conn)
    }

    fn create_table<T: Table>(self: &Self) -> Result<(), Error> {
        self.execute(T::CREATE_TABLE, &[])?;
        Ok(())
    }

    fn select<'a, T: Table>(self: &'a Self) -> Select<'a, T> {
        Select { conn: self, phantom: PhantomData }
    }

    fn insert<T: Table>(self: &Self, r: &T::TableRow) -> Result<(), Error> {
        T::do_insert(&self.update::<T>(), r)
    }

    fn update<'a, T: Table>(self: &'a Self) -> Update<'a, T> {
        Update { conn: self, phantom: PhantomData }
    }
}

// vi: ts=8 sts=4 et
