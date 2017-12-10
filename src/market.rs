use failure::Error;
use time::{Timespec, get_time};
use rusqlite::Row;
use rusqlite::types::ToSql;
use db::{DB, TableRow};

pub struct Market {
    db: DB,
    pub info: MarketRow
}

#[derive(Debug)]
pub struct MarketRow {
    version: u32,
    creation_time: Timespec
}

#[derive(Debug)]
pub struct UserRow {
    pub user_id: String,
    pub user_name: String,
    pub creation_time: Timespec
}

#[derive(Debug)]
pub struct IOURow {
    pub issuer: String,
    pub holder: String,
    pub amount: u32,
    pub creation_time: Timespec
}

#[derive(Debug)]
pub struct EntityRow {
    pub entity_id: String,
    pub entity_name: String,
    pub entity_type: String,
    pub creation_time: Timespec
}

#[derive(Debug)]
pub struct RelRow {
    pub rel_type: String,
    pub rel_from: String,
    pub rel_to: String,
    pub creation_time: Timespec
}

#[derive(Debug)]
pub struct PropRow {
    pub entity_id: String,
    pub prop_id: String,
    pub prop_value: String,
    pub creation_time: Timespec
}

#[derive(Debug)]
pub struct PredRow {
    pub pred_id: String,
    pub pred_name: String,
    pub pred_arity: u8,
    pub pred_type: String,
    pub pred_value: Option<String>,
    pub creation_time: Timespec
}

#[derive(Debug)]
pub struct DependRow {
    pub depend_type: String,
    pub depend_pred1: String,
    pub depend_pred2: String,
    pub depend_vars: String,
    pub depend_args1: String,
    pub depend_args2: String,
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

impl TableRow for UserRow {
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
        Ok(UserRow { user_id, user_name, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.user_id, &self.user_name, &self.creation_time]
    }
}

impl TableRow for IOURow {
    const TABLE_NAME : &'static str = "iou";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE iou (
            issuer          TEXT NOT NULL REFERENCES user(user_id),
            holder          TEXT NOT NULL REFERENCES user(user_id),
            amount          INTEGER NOT NULL,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO iou
            (issuer, holder, amount, creation_time)
            VALUES (?1, ?2, ?3, ?4)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let issuer = r.get_checked("issuer")?;
        let holder = r.get_checked("holder")?;
        let amount = r.get_checked("amount")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(IOURow { issuer, holder, amount, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.issuer, &self.holder, &self.amount, &self.creation_time]
    }
}

impl TableRow for EntityRow {
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
        Ok(EntityRow { entity_id, entity_name, entity_type, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.entity_id, &self.entity_name, &self.entity_type, &self.creation_time]
    }
}

impl TableRow for RelRow {
    const TABLE_NAME : &'static str = "rel";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE rel (
            rel_type        TEXT NOT NULL,
            rel_from        TEXT NOT NULL REFERENCES entity(entity_id),
            rel_to          TEXT_NOT_NULL REFERENCES entity(entity_id),
            creation_time   TEXT NOT NULL,
            PRIMARY KEY(rel_from, rel_type)
        )";

    const INSERT: &'static str =
        "INSERT INTO rel
            (rel_type, rel_from, rel_to, creation_time)
            VALUES (?1, ?2, ?3, ?4)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let rel_type = r.get_checked("rel_type")?;
        let rel_from = r.get_checked("rel_from")?;
        let rel_to = r.get_checked("rel_to")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(RelRow { rel_type, rel_from, rel_to, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.rel_type, &self.rel_from, &self.rel_to, &self.creation_time]
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

impl TableRow for PredRow {
    const TABLE_NAME : &'static str = "pred";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE pred (
            pred_id         TEXT NOT NULL PRIMARY KEY,
            pred_name       TEXT NOT NULL UNIQUE,
            pred_arity      INTEGER,
            pred_type       TEXT NOT NULL,
            pred_value      TEXT,
            creation_time   TEXT NOT NULL
        )";

    const INSERT: &'static str =
        "INSERT INTO pred
            (pred_id, pred_name, pred_arity, pred_type, pred_value, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let pred_id = r.get_checked("pred_id")?;
        let pred_name = r.get_checked("pred_name")?;
        let pred_arity = r.get_checked("pred_arity")?;
        let pred_type = r.get_checked("pred_type")?;
        let pred_value = r.get_checked("pred_value")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(PredRow { pred_id, pred_name, pred_arity, pred_type, pred_value, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.pred_id, &self.pred_name, &self.pred_arity, &self.pred_type, &self.pred_value, &self.creation_time]
    }
}

impl TableRow for DependRow {
    const TABLE_NAME : &'static str = "depend";

    const CREATE_TABLE : &'static str =
        "CREATE TABLE depend (
            depend_type     TEXT NOT NULL,
            depend_pred1    TEXT NOT NULL REFERENCES pred(pred_id),
            depend_pred2    TEXT NOT NULL REFERENCES pred(pred_id),
            depend_vars     TEXT NOT NULL,
            depend_args1    TEXT NOT NULL,
            depend_args2    TEXT NOT NULL,
            creation_time   TEXT NOT NULL,
            PRIMARY KEY(depend_type, depend_pred1, depend_pred2)
        )";

    const INSERT: &'static str =
        "INSERT INTO depend
            (depend_type, depend_pred1, depend_pred2, depend_vars, depend_args1, depend_args2, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

    fn from_row(r: &Row) -> Result<Self, Error> {
        let depend_type = r.get_checked("depend_type")?;
        let depend_pred1 = r.get_checked("depend_pred1")?;
        let depend_pred2 = r.get_checked("depend_pred2")?;
        let depend_vars = r.get_checked("depend_vars")?;
        let depend_args1 = r.get_checked("depend_args1")?;
        let depend_args2 = r.get_checked("depend_args2")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(DependRow { depend_type, depend_pred1, depend_pred2, depend_vars, depend_args1, depend_args2, creation_time })
    }

    fn to_insert_params(self: &Self) -> Vec<&ToSql> {
        vec![&self.depend_type, &self.depend_pred1, &self.depend_pred2, &self.depend_vars, &self.depend_args1, &self.depend_args2, &self.creation_time]
    }
}

impl Market {
    pub fn create_new(mut db: DB) -> Result<Market, Error> {
        db.make_table::<MarketRow>()?;
        db.make_table::<UserRow>()?;
        db.make_table::<IOURow>()?;
        db.make_table::<EntityRow>()?;
        db.make_table::<RelRow>()?;
        db.make_table::<PropRow>()?;
        db.make_table::<PredRow>()?;
        db.make_table::<DependRow>()?;

        let info = MarketRow { version: 1, creation_time: get_time() };
        db.insert_row(&info)?;

        Ok(Market { db: db, info: info })
    }

    pub fn open_existing(mut db: DB) -> Result<Market, Error> {
        let info = db.select_one::<MarketRow>()?;
        Ok(Market { db: db, info: info })
    }

    pub fn select_user_by_name(self: &mut Self, user_name: &str) -> Result<UserRow, Error> {
        self.db.select_one_where("user_name = ?1", &[&user_name])
    }

    pub fn select_all_user(self: &mut Self) -> Result<Vec<UserRow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_iou(self: &mut Self) -> Result<Vec<IOURow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_entity(self: &mut Self) -> Result<Vec<EntityRow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_entity_by_type(self: &mut Self, entity_type: &str) -> Result<Vec<EntityRow>, Error> {
        self.db.select_all_where("entity_type = ?1", &[&entity_type])
    }

    pub fn select_all_rel(self: &mut Self) -> Result<Vec<RelRow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_prop(self: &mut Self) -> Result<Vec<PropRow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_pred(self: &mut Self) -> Result<Vec<PredRow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_depend(self: &mut Self) -> Result<Vec<DependRow>, Error> {
        self.db.select_all()
    }

    pub fn insert_user(self: &mut Self, user: &UserRow) -> Result<(), Error> {
        self.db.insert_row(user)
    }

    pub fn insert_iou(self: &mut Self, iou: &IOURow) -> Result<(), Error> {
        self.db.insert_row(iou)
    }

    pub fn insert_entity(self: &mut Self, entity: &EntityRow) -> Result<(), Error> {
        self.db.insert_row(entity)
    }

    pub fn insert_rel(self: &mut Self, rel: &RelRow) -> Result<(), Error> {
        self.db.insert_row(rel)
    }

    pub fn insert_prop(self: &mut Self, prop: &PropRow) -> Result<(), Error> {
        self.db.insert_row(prop)
    }

    pub fn insert_pred(self: &mut Self, pred: &PredRow) -> Result<(), Error> {
        self.db.insert_row(pred)
    }

    pub fn insert_depend(self: &mut Self, depend: &DependRow) -> Result<(), Error> {
        self.db.insert_row(depend)
    }
}

