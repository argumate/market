use failure::{err_msg, Error};
use time::{get_time, Timespec};

use rusqlite;
use rusqlite::types::{FromSql, ToSql, ToSqlOutput, Value, ValueRef};
use rusqlite::Row;

use crate::db::{Select, Table, Update};
use crate::market::types::{
    ArgList, Cond, Depend, Dollars, Entity, Identity, Offer, OfferDetails, Pred, Rel, Timesecs,
    User, ID, IOU,
};

pub struct MarketTable {}
pub struct UserTable {}
pub struct IdentityTable {}
pub struct IOUTable {}
pub struct CondTable {}
pub struct OfferTable {}
pub struct EntityTable {}
pub struct RelTable {}
pub struct PropTable {}
pub struct PredTable {}
pub struct DependTable {}

#[derive(Debug)]
pub struct MarketRow {
    pub version: u32,
    pub creation_time: Timespec,
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

impl ToSql for Timesecs {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Integer(i64::from(self))))
    }
}

impl FromSql for Timesecs {
    fn column_result(value: ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        let i: i64 = FromSql::column_result(value)?;
        Ok(Timesecs::from(i))
    }
}

impl ToSql for Dollars {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.to_millibucks())))
    }
}

impl FromSql for Dollars {
    fn column_result(value: ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        let i: i64 = FromSql::column_result(value)?;
        Ok(Dollars::from_millibucks(i))
    }
}

impl ToSql for ArgList {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Text(String::from(self))))
    }
}

impl FromSql for ArgList {
    fn column_result(value: ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(ArgList::from(s.as_str()))
    }
}

#[derive(Debug)]
pub struct Record<T> {
    pub id: ID,
    pub fields: T,
    pub creation_time: Timespec,
}

impl<T> Record<T> {
    pub fn new(t: T) -> Record<T> {
        Record {
            id: ID::new(),
            fields: t,
            creation_time: get_time(),
        }
    }
}

#[derive(Debug)]
pub struct PropRow {
    pub entity_id: ID,
    pub prop_id: String,
    pub prop_value: String,
    pub creation_time: Timespec,
}

impl Table for MarketTable {
    type TableRow = MarketRow;

    const TABLE_NAME: &'static str = "market";

    const CREATE_TABLE: &'static str = "CREATE TABLE market (
            version         INTEGER NOT NULL,
            creation_time   TEXT NOT NULL
        )";

    fn from_row(r: &Row) -> Result<MarketRow, Error> {
        let version = r.get_checked("version")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(MarketRow {
            version,
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(version, creation_time)
            VALUES (?1, ?2)",
            &[&r.version, &r.creation_time],
        )
    }
}

impl Table for UserTable {
    type TableRow = Record<User>;

    const TABLE_NAME: &'static str = "user";

    const CREATE_TABLE: &'static str = "CREATE TABLE user (
            user_id             TEXT NOT NULL PRIMARY KEY,
            user_name           TEXT NOT NULL UNIQUE,
            user_name_stripped  TEXT NOT NULL UNIQUE,
            user_locked         BOOLEAN,
            creation_time       TEXT NOT NULL
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let user_id = r.get_checked("user_id")?;
        let user_name = r.get_checked("user_name")?;
        let user_locked = r.get_checked("user_locked")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: user_id,
            fields: User {
                user_name,
                user_locked,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(user_id, user_name, user_name_stripped, user_locked, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                &r.id,
                &r.fields.user_name,
                &User::user_name_stripped(&r.fields.user_name),
                &r.fields.user_locked,
                &r.creation_time,
            ],
        )
    }
}

impl<'a> Select<'a, UserTable> {
    pub fn by_id(&self, id: &ID) -> Result<Record<User>, Error> {
        self.one_where("user_id = ?1", &[id])
    }
}

impl<'a> Select<'a, UserTable> {
    pub fn by_user_name(&self, user_name: &str) -> Result<Record<User>, Error> {
        self.one_where("user_name = ?1", &[&user_name])
    }
}

impl<'a> Select<'a, UserTable> {
    pub fn by_user_name_stripped(&self, user_name_stripped: &str) -> Result<Record<User>, Error> {
        self.one_where("user_name_stripped = ?1", &[&user_name_stripped])
    }
}

impl Table for IdentityTable {
    type TableRow = Record<Identity>;

    const TABLE_NAME: &'static str = "identity";

    const CREATE_TABLE: &'static str = "CREATE TABLE identity (
            identity_id             TEXT NOT NULL PRIMARY KEY,
            identity_user_id        TEXT NOT NULL REFERENCES user(user_id),
            identity_service        TEXT NOT NULL,
            identity_account_name   TEXT NOT NULL,
            identity_attested_time  INTEGER NOT NULL,
            creation_time           TEXT NOT NULL,
            UNIQUE(identity_user_id, identity_service)
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let identity_id = r.get_checked("identity_id")?;
        let identity_user_id = r.get_checked("identity_user_id")?;
        let identity_service = r.get_checked("identity_service")?;
        let identity_account_name = r.get_checked("identity_account_name")?;
        let identity_attested_time = r.get_checked("identity_attested_time")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: identity_id,
            fields: Identity {
                identity_user_id,
                identity_service,
                identity_account_name,
                identity_attested_time,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(identity_id, identity_user_id, identity_service, identity_account_name, identity_attested_time, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &r.id,
                &r.fields.identity_user_id,
                &r.fields.identity_service,
                &r.fields.identity_account_name,
                &r.fields.identity_attested_time,
                &r.creation_time,
            ],
        )
    }
}

impl Table for IOUTable {
    type TableRow = Record<IOU>;

    const TABLE_NAME: &'static str = "iou";

    const CREATE_TABLE: &'static str = "CREATE TABLE iou (
            iou_id          TEXT NOT NULL PRIMARY KEY,
            iou_issuer      TEXT NOT NULL REFERENCES user(user_id),
            iou_holder      TEXT NOT NULL REFERENCES user(user_id),
            iou_value       INTEGER NOT NULL,
            iou_cond_id     TEXT REFERENCES cond(cond_id),
            iou_cond_flag   INTEGER NOT NULL,
            iou_cond_time   INTEGER,
            iou_split       TEXT REFERENCES iou(iou_id),
            iou_void        BOOLEAN,
            creation_time   TEXT NOT NULL
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let iou_id = r.get_checked("iou_id")?;
        let iou_issuer = r.get_checked("iou_issuer")?;
        let iou_holder = r.get_checked("iou_holder")?;
        let iou_value = r.get_checked("iou_value")?;
        let iou_cond_id = r.get_checked("iou_cond_id")?;
        let iou_cond_flag = r.get_checked("iou_cond_flag")?;
        let iou_cond_time = r.get_checked("iou_cond_time")?;
        let iou_split = r.get_checked("iou_split")?;
        let iou_void = r.get_checked("iou_void")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: iou_id,
            fields: IOU {
                iou_issuer,
                iou_holder,
                iou_value,
                iou_cond_id,
                iou_cond_flag,
                iou_cond_time,
                iou_split,
                iou_void,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(iou_id, iou_issuer, iou_holder, iou_value, iou_cond_id, iou_cond_flag, iou_cond_time, iou_split, iou_void, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            &[
                &r.id,
                &r.fields.iou_issuer,
                &r.fields.iou_holder,
                &r.fields.iou_value,
                &r.fields.iou_cond_id,
                &r.fields.iou_cond_flag,
                &r.fields.iou_cond_time,
                &r.fields.iou_split,
                &r.fields.iou_void,
                &r.creation_time
            ])
    }
}

impl<'a> Select<'a, IOUTable> {
    pub fn by_id(&self, id: &ID) -> Result<Record<IOU>, Error> {
        self.one_where("iou_id = ?1", &[id])
    }
}

impl<'a> Update<'a, IOUTable> {
    pub fn void_iou(&self, id: &ID) -> Result<(), Error> {
        self.update_one("iou_void = 1 WHERE iou_id = ?1 AND iou_void = 0", &[id])
    }
}

impl Table for CondTable {
    type TableRow = Record<Cond>;

    const TABLE_NAME: &'static str = "cond";

    const CREATE_TABLE: &'static str = "CREATE TABLE cond (
            cond_id         TEXT NOT NULL PRIMARY KEY,
            cond_pred       TEXT NOT NULL REFERENCES pred(pred_id),
            cond_arg1       TEXT REFERENCES entity(entity_id),
            cond_arg2       TEXT REFERENCES entity(entity_id),
            creation_time   TEXT NOT NULL
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let cond_id = r.get_checked("cond_id")?;
        let cond_pred = r.get_checked("cond_pred")?;
        let cond_arg1 = r.get_checked("cond_arg1")?;
        let cond_arg2 = r.get_checked("cond_arg2")?;
        let creation_time = r.get_checked("creation_time")?;
        let mut cond_args = Vec::new();
        if let Some(arg1) = cond_arg1 {
            cond_args.push(arg1);
            if let Some(arg2) = cond_arg2 {
                cond_args.push(arg2);
            }
        }
        Ok(Record {
            id: cond_id,
            fields: Cond {
                cond_pred,
                cond_args,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        let cond_args = &r.fields.cond_args;
        if cond_args.len() <= 2 {
            let cond_arg1 = if cond_args.len() > 0 {
                Some(cond_args[0].clone())
            } else {
                None
            };
            let cond_arg2 = if cond_args.len() > 1 {
                Some(cond_args[1].clone())
            } else {
                None
            };
            table.insert(
                "(cond_id, cond_pred, cond_arg1, cond_arg2, creation_time)
                VALUES (?1, ?2, ?3, ?4, ?5)",
                &[
                    &r.id,
                    &r.fields.cond_pred,
                    &cond_arg1,
                    &cond_arg2,
                    &r.creation_time,
                ],
            )
        } else {
            Err(err_msg(format!(
                "cond has too many arguments: {}",
                cond_args.len()
            )))
        }
    }
}

impl Table for OfferTable {
    type TableRow = Record<Offer>;

    const TABLE_NAME: &'static str = "offer";

    const CREATE_TABLE: &'static str = "CREATE TABLE offer (
            offer_id            TEXT NOT NULL PRIMARY KEY,
            offer_user          TEXT NOT NULL REFERENCES user(user_id),
            offer_cond_id       TEXT NOT NULL REFERENCES cond(cond_id),
            offer_cond_time     INTEGER,
            offer_buy_price     INTEGER NOT NULL,
            offer_sell_price    INTEGER NOT NULL,
            offer_buy_quantity    INTEGER NOT NULL,
            offer_sell_quantity   INTEGER NOT NULL,
            creation_time       TEXT NOT NULL,
            UNIQUE(offer_user, offer_cond_id, offer_cond_time)
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let offer_id = r.get_checked("offer_id")?;
        let offer_user = r.get_checked("offer_user")?;
        let offer_cond_id = r.get_checked("offer_cond_id")?;
        let offer_cond_time = r.get_checked("offer_cond_time")?;
        let offer_buy_price = r.get_checked("offer_buy_price")?;
        let offer_sell_price = r.get_checked("offer_sell_price")?;
        let offer_buy_quantity = r.get_checked("offer_buy_quantity")?;
        let offer_sell_quantity = r.get_checked("offer_sell_quantity")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: offer_id,
            fields: Offer {
                offer_user,
                offer_cond_id,
                offer_cond_time,
                offer_details: OfferDetails {
                    offer_buy_price,
                    offer_sell_price,
                    offer_buy_quantity,
                    offer_sell_quantity,
                },
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(offer_id, offer_user, offer_cond_id, offer_cond_time, offer_buy_price, offer_sell_price, offer_buy_quantity, offer_sell_quantity, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                &r.id,
                &r.fields.offer_user,
                &r.fields.offer_cond_id,
                &r.fields.offer_cond_time,
                &r.fields.offer_details.offer_buy_price,
                &r.fields.offer_details.offer_sell_price,
                &r.fields.offer_details.offer_buy_quantity,
                &r.fields.offer_details.offer_sell_quantity,
                &r.creation_time
            ])
    }
}

impl<'a> Update<'a, OfferTable> {
    pub fn update_offer(&self, id: &ID, offer: &OfferDetails) -> Result<(), Error> {
        self.update_one(
            "offer_buy_price = ?2, offer_sell_price = ?3,
            offer_buy_quantity = ?4, offer_sell_quantity = ?5
            WHERE offer_id = ?1",
            &[
                id,
                &offer.offer_buy_price,
                &offer.offer_sell_price,
                &offer.offer_buy_quantity,
                &offer.offer_sell_quantity,
            ],
        )
    }
}

impl Table for EntityTable {
    type TableRow = Record<Entity>;

    const TABLE_NAME: &'static str = "entity";

    const CREATE_TABLE: &'static str = "CREATE TABLE entity (
            entity_id       TEXT NOT NULL PRIMARY KEY,
            entity_name     TEXT NOT NULL UNIQUE,
            entity_type     TEXT NOT NULL,
            creation_time   TEXT NOT NULL
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let entity_id = r.get_checked("entity_id")?;
        let entity_name = r.get_checked("entity_name")?;
        let entity_type = r.get_checked("entity_type")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: entity_id,
            fields: Entity {
                entity_name,
                entity_type,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(entity_id, entity_name, entity_type, creation_time)
            VALUES (?1, ?2, ?3, ?4)",
            &[
                &r.id,
                &r.fields.entity_name,
                &r.fields.entity_type,
                &r.creation_time,
            ],
        )
    }
}

impl<'a> Select<'a, EntityTable> {
    pub fn by_entity_type(&self, entity_type: &str) -> Result<Vec<Record<Entity>>, Error> {
        self.all_where("entity_type = ?1", &[&entity_type])
    }
}

impl Table for RelTable {
    type TableRow = Record<Rel>;

    const TABLE_NAME: &'static str = "rel";

    const CREATE_TABLE: &'static str = "CREATE TABLE rel (
            rel_id          TEXT NOT NULL PRIMARY KEY,
            rel_type        TEXT NOT NULL,
            rel_from        TEXT NOT NULL REFERENCES entity(entity_id),
            rel_to          TEXT_NOT_NULL REFERENCES entity(entity_id),
            creation_time   TEXT NOT NULL,
            UNIQUE(rel_from, rel_type)
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let rel_id = r.get_checked("rel_id")?;
        let rel_type = r.get_checked("rel_type")?;
        let rel_from = r.get_checked("rel_from")?;
        let rel_to = r.get_checked("rel_to")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: rel_id,
            fields: Rel {
                rel_type,
                rel_from,
                rel_to,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(rel_id, rel_type, rel_from, rel_to, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                &r.id,
                &r.fields.rel_type,
                &r.fields.rel_from,
                &r.fields.rel_to,
                &r.creation_time,
            ],
        )
    }
}

impl Table for PropTable {
    type TableRow = PropRow;

    const TABLE_NAME: &'static str = "prop";

    const CREATE_TABLE: &'static str = "CREATE TABLE prop (
            entity_id       TEXT NOT NULL REFERENCES entity(entity_id),
            prop_id         TEXT NOT NULL,
            prop_value      TEXT_NOT_NULL,
            creation_time   TEXT NOT NULL,
            PRIMARY KEY(entity_id, prop_id)
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let entity_id = r.get_checked("entity_id")?;
        let prop_id = r.get_checked("prop_id")?;
        let prop_value = r.get_checked("prop_value")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(PropRow {
            entity_id,
            prop_id,
            prop_value,
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(entity_id, prop_id, prop_value, creation_time)
            VALUES (?1, ?2, ?3, ?4)",
            &[&r.entity_id, &r.prop_id, &r.prop_value, &r.creation_time],
        )
    }
}

impl Table for PredTable {
    type TableRow = Record<Pred>;

    const TABLE_NAME: &'static str = "pred";

    const CREATE_TABLE: &'static str = "CREATE TABLE pred (
            pred_id         TEXT NOT NULL PRIMARY KEY,
            pred_name       TEXT NOT NULL UNIQUE,
            pred_args       TEXT NOT NULL,
            pred_value      TEXT,
            creation_time   TEXT NOT NULL
        )";

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
        let pred_id = r.get_checked("pred_id")?;
        let pred_name = r.get_checked("pred_name")?;
        let pred_args = r.get_checked("pred_args")?;
        let pred_value = r.get_checked("pred_value")?;
        let creation_time = r.get_checked("creation_time")?;
        Ok(Record {
            id: pred_id,
            fields: Pred {
                pred_name,
                pred_args,
                pred_value,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(pred_id, pred_name, pred_args, pred_value, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                &r.id,
                &r.fields.pred_name,
                &r.fields.pred_args,
                &r.fields.pred_value,
                &r.creation_time,
            ],
        )
    }
}

impl Table for DependTable {
    type TableRow = Record<Depend>;

    const TABLE_NAME: &'static str = "depend";

    const CREATE_TABLE: &'static str = "CREATE TABLE depend (
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

    fn from_row(r: &Row) -> Result<Self::TableRow, Error> {
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
            fields: Depend {
                depend_type,
                depend_pred1,
                depend_pred2,
                depend_vars,
                depend_args1,
                depend_args2,
            },
            creation_time,
        })
    }

    fn do_insert(table: &Update<Self>, r: &Self::TableRow) -> Result<(), Error> {
        table.insert(
            "(depend_id, depend_type, depend_pred1, depend_pred2, depend_vars, depend_args1, depend_args2, creation_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            &[
                &r.id,
                &r.fields.depend_type,
                &r.fields.depend_pred1,
                &r.fields.depend_pred2,
                &r.fields.depend_vars,
                &r.fields.depend_args1,
                &r.fields.depend_args2,
                &r.creation_time
            ])
    }
}

// vi: ts=8 sts=4 et
