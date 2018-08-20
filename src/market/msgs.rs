use std::collections::HashMap;

use market::types::{Cond, Depend, Entity, Identity, Offer, OfferDetails, Pred, Rel, Transfer, User,
                    ID, IOU};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Create(Item),
    Update { id: ID, item_update: ItemUpdate },
    Query(Query),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Item {
    User(User),
    Identity(Identity),
    IOU(IOU),
    Cond(Cond),
    Offer(Offer),
    Entity(Entity),
    Rel(Rel),
    Pred(Pred),
    Depend(Depend),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemUpdate {
    Offer(OfferDetails),
    Transfer(Transfer),
    Void,
}

#[derive(Serialize, Deserialize)]
pub enum Query {
    AllUser,
    AllIOU,
    AllCond,
    AllOffer,
    AllEntity,
    AllRel,
    AllPred,
    AllDepend,
}

#[derive(Serialize)]
pub enum Error {
    InvalidUserName,
    CannotCreateUser,
    InvalidOfferDetails,
}

#[derive(Serialize)]
pub enum Response {
    Created(ID),
    Updated,
    Items(HashMap<ID, Item>),
    Error(Error),
}

pub fn single_item<T: ToItem>(id: ID, t: T) -> HashMap<ID, Item> {
    let mut items = HashMap::new();
    items.insert(id, t.to_item());
    items
}

pub trait ToItem {
    fn to_item(self) -> Item;
}

impl ToItem for User {
    fn to_item(self) -> Item {
        Item::User(self)
    }
}

impl ToItem for IOU {
    fn to_item(self) -> Item {
        Item::IOU(self)
    }
}

impl ToItem for Cond {
    fn to_item(self) -> Item {
        Item::Cond(self)
    }
}

impl ToItem for Offer {
    fn to_item(self) -> Item {
        Item::Offer(self)
    }
}

impl ToItem for Entity {
    fn to_item(self) -> Item {
        Item::Entity(self)
    }
}

impl ToItem for Rel {
    fn to_item(self) -> Item {
        Item::Rel(self)
    }
}

impl ToItem for Pred {
    fn to_item(self) -> Item {
        Item::Pred(self)
    }
}

impl ToItem for Depend {
    fn to_item(self) -> Item {
        Item::Depend(self)
    }
}

// vi: ts=8 sts=4 et
