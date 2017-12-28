use std::collections::HashMap;

use market::types::{ID, User, IOU, Cond, Offer, Entity, Rel, Pred, Depend};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Create(Item),
    Query(Query)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Item {
    User(User),
    IOU(IOU),
    Cond(Cond),
    Offer(Offer),
    Entity(Entity),
    Rel(Rel),
    Pred(Pred),
    Depend(Depend)
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
    AllDepend
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Created(ID),
    Items(HashMap<ID, Item>)
}

pub trait ToItem {
    fn to_item(self: Self) -> Item;
}

impl ToItem for User {
    fn to_item(self: Self) -> Item {
        Item::User(self)
    }
}

impl ToItem for IOU {
    fn to_item(self: Self) -> Item {
        Item::IOU(self)
    }
}

impl ToItem for Cond {
    fn to_item(self: Self) -> Item {
        Item::Cond(self)
    }
}

impl ToItem for Offer {
    fn to_item(self: Self) -> Item {
        Item::Offer(self)
    }
}

impl ToItem for Entity {
    fn to_item(self: Self) -> Item {
        Item::Entity(self)
    }
}

impl ToItem for Rel {
    fn to_item(self: Self) -> Item {
        Item::Rel(self)
    }
}

impl ToItem for Pred {
    fn to_item(self: Self) -> Item {
        Item::Pred(self)
    }
}

impl ToItem for Depend {
    fn to_item(self: Self) -> Item {
        Item::Depend(self)
    }
}

// vi: ts=8 sts=4 et
