extern crate failure;
extern crate getopts;
extern crate time;
extern crate rusqlite;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate uuid;

pub mod db;
pub mod market;

use failure::Error;
use getopts::Options;
use std::env;

use db::DB;
use market::Market;
use market::types::{ID, ArgList, UserFields, IOUFields, EntityFields, RelFields, PredFields, DependFields};
use market::msgs::{Request, Response, Item, Query};

enum CmdLine {
    Help,
    Command(Command),
}

#[derive(Debug)]
enum Command {
    Init(String),
    Status(String),
}

fn parse_command_line(opts: &Options, args: &Vec<String>) -> Result<CmdLine, Error> {
    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("h") {
        return Ok(CmdLine::Help);
    }
    let file = match matches.opt_str("f") {
        None => {
            return Err(failure::err_msg("missing --file argument"));
        }
        Some(f) => f
    };
    let cmd = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        return Err(failure::err_msg("missing command"));
    };
    match cmd.as_ref() {
        "init" => Ok(CmdLine::Command(Command::Init(file))),
        "status" => Ok(CmdLine::Command(Command::Status(file))),
        _ => Err(failure::err_msg("unknown command"))
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] CMD", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("f", "file", "set database filename", "FILE");

    match parse_command_line(&opts, &args) {
        Ok(c) => {
            match c {
                CmdLine::Help => print_usage(&program, opts),
                CmdLine::Command(cmd) => {
                    match do_command(cmd) {
                        Ok(()) => { }
                        Err(e) => println!("{}", e)
                    }
                }
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    };
}

impl Response {
    fn unwrap_id(self: Response) -> ID {
        match self {
            Response::Created(id) => id,
            Response::Items(_) => panic!("expected ID!")
        }
    }
}

fn do_command(cmd: Command) -> Result<(), Error> {
    match cmd {
        Command::Init(file) => {
            let db = DB::open_read_write(&file)?;
            let mut market = Market::create_new(db)?;

            let mrfoo = market.do_request(Request::Create(
                Item::User(UserFields {
                    user_name: String::from("Mr Foo")
                })))?.unwrap_id();

            let mrbar = market.do_request(Request::Create(
                Item::User(UserFields {
                    user_name: String::from("Mr Bar")
                })))?.unwrap_id();

            market.do_request(Request::Create(
                Item::IOU(IOUFields {
                    iou_issuer: mrfoo,
                    iou_holder: mrbar,
                    iou_amount: 17
                })))?;

            let trump = market.do_request(Request::Create(
                Item::Entity(EntityFields {
                    entity_name: String::from("Donald Trump"),
                    entity_type: String::from("person"),
                })))?.unwrap_id();

            let jeb = market.do_request(Request::Create(
                Item::Entity(EntityFields {
                    entity_name: String::from("Jeb Bush"),
                    entity_type: String::from("person"),
                })))?.unwrap_id();

            let repub = market.do_request(Request::Create(
                Item::Entity(EntityFields {
                    entity_name: String::from("Republican Party"),
                    entity_type: String::from("party"),
                })))?.unwrap_id();

            let _dem = market.do_request(Request::Create(
                Item::Entity(EntityFields {
                    entity_name: String::from("Democratic Party"),
                    entity_type: String::from("party"),
                })))?.unwrap_id();

            market.do_request(Request::Create(
                Item::Rel(RelFields {
                    rel_type: String::from("party"),
                    rel_from: jeb,
                    rel_to: repub.clone(),
                })))?;

            market.do_request(Request::Create(
                Item::Rel(RelFields {
                    rel_type: String::from("party"),
                    rel_from: trump,
                    rel_to: repub,
                })))?;

            let nominee2020 = market.do_request(Request::Create(
                Item::Pred(PredFields {
                    pred_name: String::from("Party nominee for 2020 election"),
                    pred_args: ArgList::from("party,person"),
                    pred_value: None
                })))?.unwrap_id();

            let candidate2020 = market.do_request(Request::Create(
                Item::Pred(PredFields {
                    pred_name: String::from("Candidate wins 2020 election"),
                    pred_args: ArgList::from("person"),
                    pred_value: None
                })))?.unwrap_id();

            let party2020 = market.do_request(Request::Create(
                Item::Pred(PredFields {
                    pred_name: String::from("Party wins 2020 election"),
                    pred_args: ArgList::from("party"),
                    pred_value: None
                })))?.unwrap_id();

            market.do_request(Request::Create(
                Item::Depend(DependFields {
                    depend_type: String::from("requires"),
                    depend_pred1: candidate2020.clone(),
                    depend_pred2: nominee2020,
                    depend_vars: ArgList::from("x"),
                    depend_args1: ArgList::from("x"),
                    depend_args2: ArgList::from("x.party, x")
                })))?;

            market.do_request(Request::Create(
                Item::Depend(DependFields {
                    depend_type: String::from("implies"),
                    depend_pred1: candidate2020,
                    depend_pred2: party2020,
                    depend_vars: ArgList::from("x"),
                    depend_args1: ArgList::from("x"),
                    depend_args2: ArgList::from("x.party")
                })))?;

            market.do_request(Request::Create(
                Item::Pred(PredFields {
                    pred_name: String::from("Atmospheric CO2 levels pass 500ppm"),
                    pred_args: ArgList::from("time"),
                    pred_value: None
                })))?;

            Ok(())
        }
        Command::Status(file) => {
            let db = DB::open_read_only(&file)?;
            let mut market = Market::open_existing(db)?;
            println!("{:?}", market.info);
            print_response(&market.do_request(Request::Query(Query::AllUser))?);
            print_response(&market.do_request(Request::Query(Query::AllIOU))?);
            print_response(&market.do_request(Request::Query(Query::AllEntity))?);
            print_response(&market.do_request(Request::Query(Query::AllRel))?);
            print_response(&market.do_request(Request::Query(Query::AllPred))?);
            print_response(&market.do_request(Request::Query(Query::AllDepend))?);
            Ok(())
        }
    }
}

fn print_response(response: &Response) {
    println!("{}", serde_json::to_string(response).unwrap())
}

// vi: ts=8 sts=4 et
