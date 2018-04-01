use std::collections::HashMap;
use std::ops::{Add, AddAssign, Sub};
use time::Timespec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ID(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
/// measured in millidollars
pub struct Dollars(i64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// UNIX time, seconds since 1970
pub struct Timesecs(i64);

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgList(Vec<String>);

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_name: String,
    pub user_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub identity_user_id: ID,
    pub identity_service: String,
    pub identity_account_name: String,
    pub identity_attested_time: Timesecs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOU {
    pub iou_issuer: ID,
    pub iou_holder: ID,
    pub iou_value: Dollars,
    pub iou_cond_id: Option<ID>,
    pub iou_cond_flag: bool,
    pub iou_cond_time: Option<Timesecs>,
    pub iou_split: Option<ID>,
    pub iou_void: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub holders: HashMap<ID, Dollars>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cond {
    pub cond_pred: ID,
    pub cond_args: Vec<ID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer {
    pub offer_user: ID,
    pub offer_cond_id: ID,
    pub offer_cond_time: Option<Timesecs>,
    pub offer_buy_price: Dollars,
    pub offer_sell_price: Dollars,
    pub offer_buy_quantity: u32,
    pub offer_sell_quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferUpdate {
    pub offer_buy_price: Dollars,
    pub offer_sell_price: Dollars,
    pub offer_buy_quantity: u32,
    pub offer_sell_quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub entity_name: String,
    pub entity_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rel {
    pub rel_type: String,
    pub rel_from: ID,
    pub rel_to: ID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pred {
    pub pred_name: String,
    pub pred_args: ArgList,
    pub pred_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Depend {
    pub depend_type: String,
    pub depend_pred1: ID,
    pub depend_pred2: ID,
    pub depend_vars: ArgList,
    pub depend_args1: ArgList,
    pub depend_args2: ArgList,
}

impl User {
    pub fn valid_user_name(user_name: &str) -> bool {
        user_name.chars().all(User::valid_user_name_char)
    }

    fn valid_user_name_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
    }

    pub fn user_name_stripped(user_name: &str) -> String {
        // FIXME can use string_retain on nightly
        user_name.chars().filter(char::is_ascii_alphanumeric).collect()
    }
}

impl Dollars {
    pub const ZERO: Self = Dollars(0);

    pub fn from_millibucks(m: i64) -> Self {
        Dollars(m)
    }

    pub fn to_millibucks(&self) -> i64 {
        self.0
    }
}

impl Add for Dollars {
    type Output = Dollars;

    fn add(self, other: Dollars) -> Dollars {
        Dollars(self.0 + other.0)
    }
}

impl AddAssign for Dollars {
    fn add_assign(&mut self, other: Dollars) {
        self.0 += other.0
    }
}

impl Sub for Dollars {
    type Output = Dollars;

    fn sub(self, other: Dollars) -> Dollars {
        Dollars(self.0 - other.0)
    }
}

impl<'a> From<&'a Timesecs> for Timespec {
    fn from(t: &Timesecs) -> Timespec {
        Timespec::new(t.0, 0)
    }
}

impl<'a> From<&'a Timesecs> for i64 {
    fn from(t: &Timesecs) -> i64 {
        t.0
    }
}

impl From<i64> for Timesecs {
    fn from(t: i64) -> Timesecs {
        Timesecs(t)
    }
}

impl<'a> From<&'a ArgList> for String {
    fn from(t: &ArgList) -> String {
        t.0.join(",")
    }
}

impl<'a> From<&'a str> for ArgList {
    fn from(s: &str) -> Self {
        if s.trim().is_empty() {
            ArgList(vec![])
        } else {
            ArgList(s.split(',').map(|t| t.trim().to_string()).collect())
        }
    }
}

#[test]
fn token_list_empty() {
    assert_eq!(ArgList::from("").0.len(), 0);
    assert_eq!(ArgList::from(" ").0.len(), 0);
    assert_eq!(ArgList::from(" \n\t ").0.len(), 0);
}

#[test]
fn token_list_one() {
    assert_eq!(ArgList::from("x").0.len(), 1);
    assert_eq!(ArgList::from(" x ").0.len(), 1);
}

#[test]
fn token_list_two() {
    assert_eq!(ArgList::from("x,y").0.len(), 2);
    assert_eq!(ArgList::from("x,").0.len(), 2);
    assert_eq!(ArgList::from(",y").0.len(), 2);
}

#[test]
fn dollars_ord() {
    assert!(Dollars::from_millibucks(1) > Dollars::ZERO);
    assert!(Dollars::from_millibucks(-1) < Dollars::ZERO);
    assert!(Dollars::from_millibucks(0) == Dollars::ZERO);
}

// vi: ts=8 sts=4 et
