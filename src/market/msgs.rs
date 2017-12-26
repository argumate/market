use std::collections::HashMap;

use market::types::{ID, UserFields, IOUFields, EntityFields, RelFields, PredFields, DependFields};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Create(Item),
    Query(Query)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Item {
    User(UserFields),
    IOU(IOUFields),
    Entity(EntityFields),
    Rel(RelFields),
    Pred(PredFields),
    Depend(DependFields)
}

#[derive(Serialize, Deserialize)]
pub enum Query {
    AllUser,
    AllIOU,
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

impl ToItem for UserFields {
    fn to_item(self: Self) -> Item {
        Item::User(self)
    }
}

impl ToItem for IOUFields {
    fn to_item(self: Self) -> Item {
        Item::IOU(self)
    }
}

impl ToItem for EntityFields {
    fn to_item(self: Self) -> Item {
        Item::Entity(self)
    }
}

impl ToItem for RelFields {
    fn to_item(self: Self) -> Item {
        Item::Rel(self)
    }
}

impl ToItem for PredFields {
    fn to_item(self: Self) -> Item {
        Item::Pred(self)
    }
}

impl ToItem for DependFields {
    fn to_item(self: Self) -> Item {
        Item::Depend(self)
    }
}

// vi: ts=8 sts=4 et
