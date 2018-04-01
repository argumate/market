use std::collections::HashMap;
use failure::{err_msg, Error};
use time::get_time;
use uuid::Uuid;
use rusqlite::Connection;

pub mod types;
pub mod msgs;
mod tables;

use db::DB;
use market;
use market::types::{Cond, Depend, Dollars, Entity, Pred, Rel, Transfer, User, ID, IOU};
use market::tables::{CondTable, DependTable, EntityTable, IOUTable, MarketRow, MarketTable,
                     OfferTable, PredTable, PropRow, PropTable, Record, RelTable, UserTable};
use market::msgs::{single_item, Item, ItemUpdate, Query, Request, Response, ToItem};

pub struct Market {
    db: Connection,
    pub info: MarketRow,
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

        let info = MarketRow {
            version: 1,
            creation_time: get_time(),
        };
        db.insert::<MarketTable>(&info)?;

        Ok(Market { db: db, info: info })
    }

    pub fn open_existing(db: Connection) -> Result<Market, Error> {
        let info = db.select::<MarketTable>().one()?;
        Ok(Market { db: db, info: info })
    }

    pub fn select_user_by_name(&mut self, user_name: &str) -> Result<Record<User>, Error> {
        self.db.select::<UserTable>().by_user_name(user_name)
    }

    pub fn select_all_user(&mut self) -> Result<Vec<Record<User>>, Error> {
        self.db.select::<UserTable>().all()
    }

    pub fn select_all_iou(&mut self) -> Result<Vec<Record<IOU>>, Error> {
        self.db.select::<IOUTable>().all()
    }

    pub fn select_all_cond(&mut self) -> Result<Vec<Record<Cond>>, Error> {
        self.db.select::<CondTable>().all()
    }

    pub fn select_all_entity(&mut self) -> Result<Vec<Record<Entity>>, Error> {
        self.db.select::<EntityTable>().all()
    }

    pub fn select_all_entity_by_type(
        &mut self,
        entity_type: &str,
    ) -> Result<Vec<Record<Entity>>, Error> {
        self.db.select::<EntityTable>().by_entity_type(entity_type)
    }

    pub fn select_all_rel(&mut self) -> Result<Vec<Record<Rel>>, Error> {
        self.db.select::<RelTable>().all()
    }

    pub fn select_all_prop(&mut self) -> Result<Vec<PropRow>, Error> {
        self.db.select::<PropTable>().all()
    }

    pub fn select_all_pred(&mut self) -> Result<Vec<Record<Pred>>, Error> {
        self.db.select::<PredTable>().all()
    }

    pub fn select_all_depend(&mut self) -> Result<Vec<Record<Depend>>, Error> {
        self.db.select::<DependTable>().all()
    }

    pub fn do_create(&mut self, item: Item) -> Result<Response, Error> {
        match item {
            Item::User(user) => {
                if User::valid_user_name(&user.user_name) {
                    let user_name_stripped = User::user_name_stripped(&user.user_name);
                    if let Ok(_) = self.db
                        .select::<UserTable>()
                        .by_user_name_stripped(&user_name_stripped)
                    {
                        // user_name must still be unique without punctuation
                        Ok(Response::Error(market::msgs::Error::CannotCreateUser))
                    } else {
                        let record = Record::new(user);
                        self.db.insert::<UserTable>(&record)?;
                        Ok(Response::Created(record.id))
                    }
                } else {
                    Ok(Response::Error(market::msgs::Error::CannotCreateUser))
                }
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

    fn do_iou_transfer(&mut self, id: ID, transfer: &Transfer) -> Result<HashMap<ID, Item>, Error> {
        let mut ious = HashMap::new();
        let tx = self.db.transaction()?;
        let r = tx.select::<IOUTable>().by_id(&id)?;
        let old_iou = r.fields;
        // FIXME access control
        if old_iou.iou_void {
            return Err(err_msg("IOU is already void"));
        } else {
            tx.update().void_iou(&id)?;
            let mut total = Dollars::ZERO;
            for (_, value) in &transfer.holders {
                if *value > Dollars::ZERO {
                    total += *value;
                } else {
                    return Err(err_msg("IOU value must be positive"));
                }
            }
            if total != old_iou.iou_value {
                return Err(err_msg("incorrect transfer value"));
            }
            for (user_id, value) in &transfer.holders {
                let new_iou = IOU {
                    iou_holder: user_id.clone(),
                    iou_value: *value,
                    iou_split: Some(id.clone()),
                    iou_void: *user_id == old_iou.iou_issuer,
                    ..old_iou.clone()
                };
                let new_record = Record::new(new_iou);
                tx.insert::<IOUTable>(&new_record)?;
                ious.insert(new_record.id, new_record.fields.to_item());
            }
        }
        tx.commit()?;
        Ok(ious)
    }

    fn do_iou_void(&mut self, id: &ID) -> Result<IOU, Error> {
        let tx = self.db.transaction()?;
        let mut r = tx.select::<IOUTable>().by_id(&id)?;
        // FIXME access control
        if r.fields.iou_void {
            return Err(err_msg("IOU is already void"));
        } else {
            tx.update().void_iou(&id)?;
            r.fields.iou_void = true;
        }
        tx.commit()?;
        Ok(r.fields)
    }

    pub fn do_update(&mut self, id: ID, item_update: ItemUpdate) -> Result<Response, Error> {
        match item_update {
            ItemUpdate::Offer(offer) => {
                // FIXME access control
                self.db.update::<OfferTable>().update_offer(&id, &offer)?;
                Ok(Response::Updated)
            }
            ItemUpdate::Transfer(transfer) => {
                let items = self.do_iou_transfer(id, &transfer)?;
                Ok(Response::Items(items))
            }
            ItemUpdate::Void => {
                let iou = self.do_iou_void(&id)?;
                Ok(Response::Items(single_item(id, iou)))
            }
        }
    }

    pub fn do_query(&mut self, query: Query) -> Result<Response, Error> {
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
                let items = self.db
                    .select::<OfferTable>()
                    .all()?
                    .into_iter()
                    .map(to_item)
                    .collect();
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

    pub fn do_request(&mut self, request: Request) -> Result<Response, Error> {
        match request {
            Request::Create(item) => self.do_create(item),
            Request::Update { id, item_update } => self.do_update(id, item_update),
            Request::Query(query) => self.do_query(query),
        }
    }
}

impl ID {
    fn new() -> ID {
        ID(Uuid::new_v4().simple().to_string())
    }
}

// vi: ts=8 sts=4 et
