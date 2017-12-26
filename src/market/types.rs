
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ID(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgList(Vec<String>);

#[derive(Debug, Serialize, Deserialize)]
pub struct UserFields {
    pub user_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IOUFields {
    pub iou_issuer: ID,
    pub iou_holder: ID,
    pub iou_amount: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityFields {
    pub entity_name: String,
    pub entity_type: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelFields {
    pub rel_type: String,
    pub rel_from: ID,
    pub rel_to: ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PredFields {
    pub pred_name: String,
    pub pred_args: ArgList,
    pub pred_value: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependFields {
    pub depend_type: String,
    pub depend_pred1: ID,
    pub depend_pred2: ID,
    pub depend_vars: ArgList,
    pub depend_args1: ArgList,
    pub depend_args2: ArgList
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
            ArgList(s.split(',').map(|t| { t.trim().to_string() }).collect())
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

// vi: ts=8 sts=4 et