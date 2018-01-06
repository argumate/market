use std::collections::HashMap;
use failure::Error;
use time::get_time;
use uuid::Uuid;
use rusqlite::Connection;

pub mod types;
pub mod msgs;
mod tables;

use db::DB;
use market::types::{ID, User, IOU, Cond, Entity, Rel, Pred, Depend};
use market::tables::{MarketRow, Record, PropRow, MarketTable, UserTable, IOUTable, CondTable, OfferTable, EntityTable, RelTable, PropTable, PredTable, DependTable};
use market::msgs::{Request, Response, Query, Item, ItemUpdate, ToItem};

pub struct Market {
    db: Connection,
    pub info: MarketRow
}

impl Market {
    pub fn create_new(db: Connection) -> Result<Market, Error> {
        db.create_table::<MarketTable>()?;
        db.create_table::<UserTable>()?;
        db.create_table::<IOUTable>()?;
        db.create_table::<CondTable>()?;
        db.create_table::<OfferTable>()?;
        db.create_table::<EntityTable>()?;
        db.create_table::<RelTable>()?;
        db.create_table::<PropTable>()?;
        db.create_table::<PredTable>()?;
        db.create_table::<DependTable>()?;

        let info = MarketRow { version: 1, creation_time: get_time() };
        db.insert::<MarketTable>(&info)?;

        Ok(Market { db: db, info: info })
    }

    pub fn open_existing(db: Connection) -> Result<Market, Error> {
        let info = db.select::<MarketTable>().one()?;
        Ok(Market { db: db, info: info })
    }

    pub fn select_user_by_name(self: &mut Self, user_name: &str) -> Result<Record<User>, Error> {
        self.db.select::<UserTable>().by_user_name(user_name)
    }

    pub fn select_all_user(self: &mut Self) -> Result<Vec<Record<User>>, Error> {
        self.db.select::<UserTable>().all()
    }

    pub fn select_all_iou(self: &mut Self) -> Result<Vec<Record<IOU>>, Error> {
        self.db.select::<IOUTable>().all()
    }

    pub fn select_all_cond(self: &mut Self) -> Result<Vec<Record<Cond>>, Error> {
        self.db.select::<CondTable>().all()
    }

    pub fn select_all_entity(self: &mut Self) -> Result<Vec<Record<Entity>>, Error> {
        self.db.select::<EntityTable>().all()
    }

    pub fn select_all_entity_by_type(self: &mut Self, entity_type: &str) -> Result<Vec<Record<Entity>>, Error> {
        self.db.select::<EntityTable>().by_entity_type(entity_type)
    }

    pub fn select_all_rel(self: &mut Self) -> Result<Vec<Record<Rel>>, Error> {
        self.db.select::<RelTable>().all()
    }

    pub fn select_all_prop(self: &mut Self) -> Result<Vec<PropRow>, Error> {
        self.db.select::<PropTable>().all()
    }

    pub fn select_all_pred(self: &mut Self) -> Result<Vec<Record<Pred>>, Error> {
        self.db.select::<PredTable>().all()
    }

    pub fn select_all_depend(self: &mut Self) -> Result<Vec<Record<Depend>>, Error> {
        self.db.select::<DependTable>().all()
    }

    pub fn do_create(self: &mut Self, item: Item) -> Result<Response, Error> {
        match item {
            Item::User(user) => {
                // FIXME validation
                let record = Record::new(user);
                self.db.insert::<UserTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::IOU(iou) => {
                // FIXME validation
                let record = Record::new(iou);
                self.db.insert::<IOUTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Cond(cond) => {
                // FIXME validation
                let record = Record::new(cond);
                self.db.insert::<CondTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Offer(offer) => {
                // FIXME validation
                let record = Record::new(offer);
                self.db.insert::<OfferTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Entity(entity) => {
                // FIXME validation
                let record = Record::new(entity);
                self.db.insert::<EntityTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Rel(rel) => {
                // FIXME validation
                let record = Record::new(rel);
                self.db.insert::<RelTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Pred(pred) => {
                // FIXME validation
                let record = Record::new(pred);
                self.db.insert::<PredTable>(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Depend(depend) => {
                // FIXME validation
                let record = Record::new(depend);
                self.db.insert::<DependTable>(&record)?;
                Ok(Response::Created(record.id))
            }
        }
    }

    pub fn do_update(self: &mut Self, items: HashMap<ID, ItemUpdate>) -> Result<Response, Error> {
        for (id, item) in items {
            match item {
                ItemUpdate::Offer(offer) => {
                    // FIXME access control
                    self.db.update::<OfferTable>().update_offer(id, &offer)?;
                }
            }
        }
        Ok(Response::Updated)
    }

    pub fn do_query(self: &mut Self, query: Query) -> Result<Response, Error> {
        fn to_item<T: ToItem>(record: Record<T>) -> (ID, Item) {
            (record.id, record.fields.to_item())
        }

        match query {
            Query::AllUser => {
                // FIXME access control
                let items = self.select_all_user()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllIOU => {
                // FIXME access control
                let items = self.select_all_iou()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllCond => {
                // FIXME access control
                let items = self.select_all_cond()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllOffer => {
                // FIXME access control
                let items = self.db.select::<OfferTable>().all()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllEntity => {
                // FIXME access control
                let items = self.select_all_entity()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllRel => {
                // FIXME access control
                let items = self.select_all_rel()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllPred => {
                // FIXME access control
                let items = self.select_all_pred()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
            Query::AllDepend => {
                // FIXME access control
                let items = self.select_all_depend()?.into_iter().map(to_item).collect();
                Ok(Response::Items(items))
            }
        }
    }

    pub fn do_request(self: &mut Self, request: Request) -> Result<Response, Error> {
        match request {
            Request::Create(item) => self.do_create(item),
            Request::Update(items) => self.do_update(items),
            Request::Query(query) => self.do_query(query)
        }
    }
}

impl ID {
    fn new() -> ID {
        ID(Uuid::new_v4().simple().to_string())
    }
}

// vi: ts=8 sts=4 et
