use failure::Error;
use time::{Timespec, get_time};

use rusqlite;
use rusqlite::Row;
use rusqlite::types::{ToSql, ToSqlOutput, FromSql, Value, ValueRef};

use db::TableRow;
use market::types::{ID, ArgList, UserFields, IOUFields, EntityFields, RelFields, PredFields, DependFields};

#[derive(Debug)]
pub struct MarketRow {
    pub version: u32,
    pub creation_time: Timespec
}

impl ToSql for ID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        ToSql::to_sql(&self.0)
    }
}

impl FromSql for ID {
    fn column_result(value: ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        let s = FromSql::column_result(value)?;
        Ok(ID(s))
    }
}

impl ToSql for ArgList {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Text(String::from(self))))
    }
}

impl FromSql for ArgList {
    fn column_result(value: ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        let s : String = FromSql::column_result(value)?;
        Ok(ArgList::from(s.as_str()))
    }
}

#[derive(Debug)]
pub struct Record<T> {
    pub id: ID,
    pub fields: T,
    pub creation_time: Timespec
}

impl<T> Record<T> {
    pub fn new(t: T) -> Record<T> {
        Record {
            id: ID::new(),
            fields: t,
            creation_time: get_time()
        }
    }
}

#[derive(Debug)]
pub struct PropRow {
    pub entity_id: ID,
    pub prop_id: String,
    pub prop_value: String,
    pub creation_time: Timespec
}

impl TableRow for MarketRow {
    const TABLE_NAME : &'static str = "market";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE market (
            version         INTEGER NOT NULL,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO market
            (version, creation_time)
            VALUES (?1, ?2)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let version = r.get_checked("version")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(MarketRow { version, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.version, &self.creation_time]
    }
}

impl TableRow for Record<UserFields> {
    const TABLE_NAME : &'static str = "user";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE user (
            user_id         TEXT NOT NULL PRIMARY KEY,
            user_name       TEXT NOT NULL UNIQUE,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO user
            (user_id, user_name, creation_time)
            VALUES (?1, ?2, ?3)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let user_id = r.get_checked("user_id")?;
        let user_name = r.get_checked("user_name")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: user_id,
            fields: UserFields {
                user_name
            },
            creation_time
        })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.id, &self.fields.user_name, &self.creation_time]
    }
}

impl TableRow for Record<IOUFields> {
    const TABLE_NAME : &'static str = "iou";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE iou (
            iou_id          TEXT NOT NULL PRIMARY KEY,
            iou_issuer      TEXT NOT NULL REFERENCES user(user_id),
            iou_holder      TEXT NOT NULL REFERENCES user(user_id),
            iou_amount      INTEGER NOT NULL,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO iou
            (iou_id, iou_issuer, iou_holder, iou_amount, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let iou_id = r.get_checked("iou_id")?;
        let iou_issuer = r.get_checked("iou_issuer")?;
        let iou_holder = r.get_checked("iou_holder")?;
        let iou_amount = r.get_checked("iou_amount")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: iou_id,
            fields: IOUFields {
                iou_issuer, iou_holder, iou_amount
            },
            creation_time
        })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.id, &self.fields.iou_issuer, &self.fields.iou_holder, &self.fields.iou_amount, &self.creation_time]
    }
}

impl TableRow for Record<EntityFields> {
    const TABLE_NAME : &'static str = "entity";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE entity (
            entity_id       TEXT NOT NULL PRIMARY KEY,
            entity_name     TEXT NOT NULL UNIQUE,
            entity_type     TEXT NOT NULL,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO entity
            (entity_id, entity_name, entity_type, creation_time)
            VALUES (?1, ?2, ?3, ?4)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let entity_id = r.get_checked("entity_id")?;
        let entity_name = r.get_checked("entity_name")?;
        let entity_type = r.get_checked("entity_type")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: entity_id,
            fields: EntityFields {
                entity_name, entity_type
            },
            creation_time
        })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.id, &self.fields.entity_name, &self.fields.entity_type, &self.creation_time]
    }
}

impl TableRow for Record<RelFields> {
    const TABLE_NAME : &'static str = "rel";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE rel (
            rel_id          TEXT NOT NULL PRIMARY KEY,
            rel_type        TEXT NOT NULL,
            rel_from        TEXT NOT NULL REFERENCES entity(entity_id),
            rel_to          TEXT_NOT_NULL REFERENCES entity(entity_id),
            creation_time   TEXT NOT NULL,
            UNIQUE(rel_from, rel_type)
        )";

    const INSERT: &'static str =
        "INSERT INTO rel
            (rel_id, rel_type, rel_from, rel_to, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let rel_id = r.get_checked("rel_id")?;
        let rel_type = r.get_checked("rel_type")?;
        let rel_from = r.get_checked("rel_from")?;
        let rel_to = r.get_checked("rel_to")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: rel_id,
            fields: RelFields {
                rel_type, rel_from, rel_to
            },
            creation_time
        })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.id, &self.fields.rel_type, &self.fields.rel_from, &self.fields.rel_to, &self.creation_time]
    }
}

impl TableRow for PropRow {
    const TABLE_NAME : &'static str = "prop";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE prop (
            entity_id       TEXT NOT NULL REFERENCES entity(entity_id),
            prop_id         TEXT NOT NULL,
            prop_value      TEXT_NOT_NULL,
            creation_time   TEXT NOT NULL,
            PRIMARY KEY(entity_id, prop_id)
        )";

    const INSERT: &'static str =
        "INSERT INTO prop
            (entity_id, prop_id, prop_value, creation_time)
            VALUES (?1, ?2, ?3, ?4)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let entity_id = r.get_checked("entity_id")?;
        let prop_id = r.get_checked("prop_id")?;
        let prop_value = r.get_checked("prop_value")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(PropRow { entity_id, prop_id, prop_value, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.entity_id, &self.prop_id, &self.prop_value, &self.creation_time]
    }
}

impl TableRow for Record<PredFields> {
    const TABLE_NAME : &'static str = "pred";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE pred (
            pred_id         TEXT NOT NULL PRIMARY KEY,
            pred_name       TEXT NOT NULL UNIQUE,
            pred_args       TEXT NOT NULL,
            pred_value      TEXT,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO pred
            (pred_id, pred_name, pred_args, pred_value, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let pred_id = r.get_checked("pred_id")?;
        let pred_name = r.get_checked("pred_name")?;
        let pred_args = r.get_checked("pred_args")?;
        let pred_value = r.get_checked("pred_value")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: pred_id,
            fields: PredFields {
                pred_name, pred_args, pred_value
            },
            creation_time
        })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.id, &self.fields.pred_name, &self.fields.pred_args, &self.fields.pred_value, &self.creation_time]
    }
}

impl TableRow for Record<DependFields> {
    const TABLE_NAME : &'static str = "depend";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE depend (
            depend_id       TEXT NOT NULL PRIMARY KEY,
            depend_type     TEXT NOT NULL,
            depend_pred1    TEXT NOT NULL REFERENCES pred(pred_id),
            depend_pred2    TEXT NOT NULL REFERENCES pred(pred_id),
            depend_vars     TEXT NOT NULL,
            depend_args1    TEXT NOT NULL,
            depend_args2    TEXT NOT NULL,
            creation_time   TEXT NOT NULL,
            UNIQUE(depend_type, depend_pred1, depend_pred2)
        )";

    const INSERT: &'static str =
        "INSERT INTO depend
            (depend_id, depend_type, depend_pred1, depend_pred2, depend_vars, depend_args1, depend_args2, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let depend_id = r.get_checked("depend_id")?;
        let depend_type = r.get_checked("depend_type")?;
        let depend_pred1 = r.get_checked("depend_pred1")?;
        let depend_pred2 = r.get_checked("depend_pred2")?;
        let depend_vars = r.get_checked("depend_vars")?;
        let depend_args1 = r.get_checked("depend_args1")?;
        let depend_args2 = r.get_checked("depend_args2")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: depend_id,
            fields: DependFields {
                depend_type, depend_pred1, depend_pred2, depend_vars, depend_args1, depend_args2
            },
            creation_time
        })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.id, &self.fields.depend_type, &self.fields.depend_pred1, &self.fields.depend_pred2, &self.fields.depend_vars, &self.fields.depend_args1, &self.fields.depend_args2, &self.creation_time]
    }
}

// vi: ts=8 sts=4 et
