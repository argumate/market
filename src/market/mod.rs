use failure::Error;
use time::get_time;
use uuid::Uuid;

pub mod types;
pub mod msgs;
mod tables;

use db::DB;
use market::types::{ID, UserFields, IOUFields, EntityFields, RelFields, PredFields, DependFields};
use market::tables::{MarketRow, Record, PropRow};
use market::msgs::{Request, Response, Query, Item, ToItem};

pub struct Market {
    db: DB,
    pub info: MarketRow
}

impl Market {
    pub fn create_new(mut db: DB) -> Result<Market, Error> {
        db.make_table::<MarketRow>()?;
        db.make_table::<Record<UserFields>>()?;
        db.make_table::<Record<IOUFields>>()?;
        db.make_table::<Record<EntityFields>>()?;
        db.make_table::<Record<RelFields>>()?;
        db.make_table::<PropRow>()?;
        db.make_table::<Record<PredFields>>()?;
        db.make_table::<Record<DependFields>>()?;

        let info = MarketRow { version: 1, creation_time: get_time() };
        db.insert_row(&info)?;

        Ok(Market { db: db, info: info })
    }

    pub fn open_existing(mut db: DB) -> Result<Market, Error> {
        let info = db.select_one::<MarketRow>()?;
        Ok(Market { db: db, info: info })
    }

    pub fn select_user_by_name(self: &mut Self, user_name: &str) -> Result<Record<UserFields>, Error> {
        self.db.select_one_where("user_name = ?1", &[&user_name])
    }

    pub fn select_all_user(self: &mut Self) -> Result<Vec<Record<UserFields>>, Error> {
        self.db.select_all()
    }

    pub fn select_all_iou(self: &mut Self) -> Result<Vec<Record<IOUFields>>, Error> {
        self.db.select_all()
    }

    pub fn select_all_entity(self: &mut Self) -> Result<Vec<Record<EntityFields>>, Error> {
        self.db.select_all()
    }

    pub fn select_all_entity_by_type(self: &mut Self, entity_type: &str) -> Result<Vec<Record<EntityFields>>, Error> {
        self.db.select_all_where("entity_type = ?1", &[&entity_type])
    }

    pub fn select_all_rel(self: &mut Self) -> Result<Vec<Record<RelFields>>, Error> {
        self.db.select_all()
    }

    pub fn select_all_prop(self: &mut Self) -> Result<Vec<PropRow>, Error> {
        self.db.select_all()
    }

    pub fn select_all_pred(self: &mut Self) -> Result<Vec<Record<PredFields>>, Error> {
        self.db.select_all()
    }

    pub fn select_all_depend(self: &mut Self) -> Result<Vec<Record<DependFields>>, Error> {
        self.db.select_all()
    }

    pub fn do_create(self: &mut Self, item: Item) -> Result<Response, Error> {
        match item {
            Item::User(user) => {
                // FIXME validation
                let record = Record::new(user);
                self.db.insert_row(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::IOU(iou) => {
                // FIXME validation
                let record = Record::new(iou);
                self.db.insert_row(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Entity(entity) => {
                // FIXME validation
                let record = Record::new(entity);
                self.db.insert_row(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Rel(rel) => {
                // FIXME validation
                let record = Record::new(rel);
                self.db.insert_row(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Pred(pred) => {
                // FIXME validation
                let record = Record::new(pred);
                self.db.insert_row(&record)?;
                Ok(Response::Created(record.id))
            }
            Item::Depend(depend) => {
                // FIXME validation
                let record = Record::new(depend);
                self.db.insert_row(&record)?;
                Ok(Response::Created(record.id))
            }
        }
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
