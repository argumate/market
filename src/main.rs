extern crate failure;
extern crate getopts;
extern crate rusqlite;
extern crate time;

pub mod db;
pub mod market;

use failure::Error;
use getopts::Options;
use std::env;
use time::get_time;

use db::DB;
use market::{Market, UserRow, IOURow, EntityRow, RelRow, /*PropRow,*/ PredRow, DependRow};

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

fn do_command(cmd: Command) -> Result<(), Error> {
    match cmd {
        Command::Init(file) => {
            let db = DB::open_read_write(&file)?;
            let mut market = Market::create_new(db)?;

            market.insert_user(&UserRow {
                    user_id: String::from("foo"),
                    user_name: String::from("Mr Foo"),
                    creation_time: get_time()
                })?;

            market.insert_user(&UserRow {
                    user_id: String::from("bar"),
                    user_name: String::from("Mr Bar"),
                    creation_time: get_time()
                })?;

            market.insert_iou(&IOURow {
                    issuer: String::from("foo"),
                    holder: String::from("bar"),
                    amount: 17,
                    creation_time: get_time()
                })?;

            market.insert_entity(&EntityRow {
                    entity_id: String::from("trump"),
                    entity_name: String::from("Donald Trump"),
                    entity_type: String::from("person"),
                    creation_time: get_time()
                })?;

            market.insert_entity(&EntityRow {
                    entity_id: String::from("jeb"),
                    entity_name: String::from("Jeb Bush"),
                    entity_type: String::from("person"),
                    creation_time: get_time()
                })?;

            market.insert_entity(&EntityRow {
                    entity_id: String::from("republican"),
                    entity_name: String::from("Republican Party"),
                    entity_type: String::from("party"),
                    creation_time: get_time()
                })?;

            market.insert_entity(&EntityRow {
                    entity_id: String::from("democrat"),
                    entity_name: String::from("Democratic Party"),
                    entity_type: String::from("party"),
                    creation_time: get_time()
                })?;

            market.insert_rel(&RelRow {
                    rel_type: String::from("party"),
                    rel_from: String::from("jeb"),
                    rel_to: String::from("republican"),
                    creation_time: get_time()
                })?;

            market.insert_rel(&RelRow {
                    rel_type: String::from("party"),
                    rel_from: String::from("trump"),
                    rel_to: String::from("republican"),
                    creation_time: get_time()
                })?;

            market.insert_pred(&PredRow {
                    pred_id: String::from("nominee2020"),
                    pred_name: String::from("Party nominee for 2020 election"),
                    pred_arity: 2,
                    pred_type: String::from("party, person"),
                    pred_value: None,
                    creation_time: get_time()
                })?;

            market.insert_pred(&PredRow {
                    pred_id: String::from("candidate2020"),
                    pred_name: String::from("Candidate wins 2020 election"),
                    pred_arity: 1,
                    pred_type: String::from("person"),
                    pred_value: None,
                    creation_time: get_time()
                })?;

            market.insert_pred(&PredRow {
                    pred_id: String::from("party2020"),
                    pred_name: String::from("Party wins 2020 election"),
                    pred_arity: 1,
                    pred_type: String::from("party"),
                    pred_value: None,
                    creation_time: get_time()
                })?;

            market.insert_depend(&DependRow {
                    depend_type: String::from("requires"),
                    depend_pred1: String::from("candidate2020"),
                    depend_pred2: String::from("nominee2020"),
                    depend_vars: String::from("x"),
                    depend_args1: String::from("x"),
                    depend_args2: String::from("x.party, x"),
                    creation_time: get_time()
                })?;

            market.insert_depend(&DependRow {
                    depend_type: String::from("implies"),
                    depend_pred1: String::from("candidate2020"),
                    depend_pred2: String::from("party2020"),
                    depend_vars: String::from("x"),
                    depend_args1: String::from("x"),
                    depend_args2: String::from("x.party"),
                    creation_time: get_time()
                })?;

            market.insert_pred(&PredRow {
                    pred_id: String::from("ppm500"),
                    pred_name: String::from("Atmospheric CO2 levels pass 500ppm"),
                    pred_arity: 1,
                    pred_type: String::from("time"),
                    pred_value: None,
                    creation_time: get_time()
                })?;

            Ok(())
        }
        Command::Status(file) => {
            let db = DB::open_read_only(&file)?;
            let mut market = Market::open_existing(db)?;
            println!("{:?}", market.info);
            for user in market.select_all_user()? {
                println!("{:?}", user);
            }
            for iou in market.select_all_iou()? {
                println!("{:?}", iou);
            }
            for entity in market.select_all_entity()? {
                println!("{:?}", entity);
            }
            for rel in market.select_all_rel()? {
                println!("{:?}", rel);
            }
            for prop in market.select_all_prop()? {
                println!("{:?}", prop);
            }
            for pred in market.select_all_pred()? {
                println!("{:?}", pred);
            }
            for depend in market.select_all_depend()? {
                println!("{:?}", depend);
            }
            Ok(())
        }
    }
}

// vi: ts=8 sts=4 et
