extern crate failure;
extern crate getopts;
extern crate rusqlite;
extern crate time;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate uuid;

extern crate actix;
extern crate actix_web;
extern crate futures;

pub mod db;
pub mod market;
pub mod server;

use failure::{err_msg, format_err, Error};
use getopts::Options;
use std::collections::HashMap;
use std::env;

use db::DB;
use market::msgs::{Item, ItemUpdate, Query, Request, Response};
use market::types::{
    ArgList, Cond, Depend, Dollars, Entity, Identity, Offer, OfferDetails, Pred, Rel, Timesecs,
    Transfer, User, ID, IOU,
};
use market::Market;
use server::run_server;

struct CmdLine {
    config: Config,
    command: Command,
}

struct Config {
    help: bool,
    db_filename: String,
    time: Timesecs,
}

enum Command {
    Usage,
    Init,
    Dummy,
    Status,
    Server(String),
    User(UserCommand),
}

enum UserCommand {
    Add(String),
}

fn parse_command_line(opts: &Options, args: &Vec<String>) -> Result<CmdLine, Error> {
    let matches = opts.parse(&args[1..])?;

    let help = matches.opt_present("h");
    let db_filename = match matches.opt_str("f") {
        None => String::from("market.db"),
        Some(f) => f,
    };
    let time = match matches.opt_str("t") {
        None => Timesecs::now(),
        Some(t) => Timesecs::parse_datetime(&t)?,
    };
    let config = Config {
        help,
        db_filename,
        time,
    };
    let command = parse_command(&matches.free)?;
    Ok(CmdLine { config, command })
}

fn parse_command(cmd: &[String]) -> Result<Command, Error> {
    if !cmd.is_empty() {
        match cmd[0].as_str() {
            "init" => parse_done(&cmd[1..], Command::Init),
            "dummy" => parse_done(&cmd[1..], Command::Dummy),
            "status" => parse_done(&cmd[1..], Command::Status),
            "server" => parse_done(&cmd[1..], Command::Server(String::from("127.0.0.1:8000"))),
            "user" => parse_user_command(&cmd[1..]),
            _ => Err(format_err!("unknown command: {}", cmd[0])),
        }
    } else {
        Ok(Command::Usage)
    }
}

fn parse_user_command(args: &[String]) -> Result<Command, Error> {
    if !args.is_empty() {
        match args[0].as_str() {
            "add" => parse_user_add_command(&args[1..]),
            _ => Err(format_err!("unknown subcommand: {}", args[0])),
        }
    } else {
        Err(err_msg("expected user subcommand [add]"))
    }
}

fn parse_user_add_command(args: &[String]) -> Result<Command, Error> {
    if !args.is_empty() {
        let command = Command::User(UserCommand::Add(args[0].clone()));
        parse_done(&args[1..], command)
    } else {
        Err(err_msg("expected user name"))
    }
}

fn parse_done(args: &[String], command: Command) -> Result<Command, Error> {
    if args.is_empty() {
        Ok(command)
    } else {
        Err(format_err!("unexpected args: {:?}", args))
    }
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [OPTIONS] COMMAND", program);
    print!("{}", opts.usage(&brief));
    println!("\nCommands:");
    println!("    init");
    println!("    dummy");
    println!("    status");
    println!("    server");
    println!("    user [add]");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print help");
    opts.optopt("f", "file", "database filename [market.db]", "FILE");
    opts.optopt("t", "time", "time of operation [current time]", "TIME");

    match main2(&opts, &args) {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn main2(opts: &Options, args: &Vec<String>) -> Result<(), Error> {
    let cmdline = parse_command_line(&opts, &args)?;
    if cmdline.config.help {
        // FIXME
    }
    match cmdline.command {
        Command::Usage => {
            let program = &args[0];
            print_usage(&program, &opts);
            Ok(())
        }
        Command::Init => init(&cmdline.config),
        Command::Dummy => dummy(&cmdline.config),
        Command::Status => status(&cmdline.config),
        Command::Server(addr) => server(&cmdline.config, &addr),
        Command::User(user_cmd) => user_command(&cmdline.config, user_cmd),
    }
}

fn user_command(config: &Config, user_cmd: UserCommand) -> Result<(), Error> {
    let db = DB::open_read_write(&config.db_filename)?;
    let mut market = Market::open_existing(db)?;
    match user_cmd {
        UserCommand::Add(user_name) => {
            let user = User {
                user_name: user_name.clone(),
                user_locked: false,
            };
            match market.do_create(Item::User(user), config.time)? {
                Ok(user_id) => {
                    println!("added user {} with id {:?}", user_name, user_id);
                    Ok(())
                }
                Err(err) => Err(format_err!("{:?}", err)),
            }
        }
    }
}

fn server(config: &Config, addr: &str) -> Result<(), Error> {
    let db = DB::open_read_write(&config.db_filename)?;
    let market = Market::open_existing(db)?;
    run_server(market, addr)
}

fn init(config: &Config) -> Result<(), Error> {
    let db = DB::open_read_write(&config.db_filename)?;
    Market::create_new(db)?;
    println!("initialised {}", config.db_filename);
    Ok(())
}

fn dummy(config: &Config) -> Result<(), Error> {
    let db = DB::open_read_write(&config.db_filename)?;
    let mut market = Market::open_existing(db)?;

    let mrfoo = market
        .do_request(Request::Create(Item::User(User {
            user_name: String::from("MrFoo"),
            user_locked: false,
        })))?
        .unwrap_id();

    let mrbar = market
        .do_request(Request::Create(Item::User(User {
            user_name: String::from("MrBar"),
            user_locked: false,
        })))?
        .unwrap_id();

    market.do_request(Request::Create(Item::Identity(Identity {
        identity_user_id: mrfoo.clone(),
        identity_service: String::from("tumblr"),
        identity_account_name: String::from("mr--foo"),
        identity_attested_time: Timesecs::from(0),
    })))?;

    let trump = market
        .do_request(Request::Create(Item::Entity(Entity {
            entity_name: String::from("Donald Trump"),
            entity_type: String::from("person"),
        })))?
        .unwrap_id();

    let jeb = market
        .do_request(Request::Create(Item::Entity(Entity {
            entity_name: String::from("Jeb Bush"),
            entity_type: String::from("person"),
        })))?
        .unwrap_id();

    let repub = market
        .do_request(Request::Create(Item::Entity(Entity {
            entity_name: String::from("Republican Party"),
            entity_type: String::from("party"),
        })))?
        .unwrap_id();

    let _dem = market
        .do_request(Request::Create(Item::Entity(Entity {
            entity_name: String::from("Democratic Party"),
            entity_type: String::from("party"),
        })))?
        .unwrap_id();

    market.do_request(Request::Create(Item::Rel(Rel {
        rel_type: String::from("party"),
        rel_from: jeb,
        rel_to: repub.clone(),
    })))?;

    market.do_request(Request::Create(Item::Rel(Rel {
        rel_type: String::from("party"),
        rel_from: trump.clone(),
        rel_to: repub,
    })))?;

    let nominee2020 = market
        .do_request(Request::Create(Item::Pred(Pred {
            pred_name: String::from("Party nominee for 2020 election"),
            pred_args: ArgList::from("party,person"),
            pred_value: None,
        })))?
        .unwrap_id();

    let candidate2020 = market
        .do_request(Request::Create(Item::Pred(Pred {
            pred_name: String::from("Candidate wins 2020 election"),
            pred_args: ArgList::from("person"),
            pred_value: None,
        })))?
        .unwrap_id();

    let party2020 = market
        .do_request(Request::Create(Item::Pred(Pred {
            pred_name: String::from("Party wins 2020 election"),
            pred_args: ArgList::from("party"),
            pred_value: None,
        })))?
        .unwrap_id();

    market.do_request(Request::Create(Item::Depend(Depend {
        depend_type: String::from("requires"),
        depend_pred1: candidate2020.clone(),
        depend_pred2: nominee2020,
        depend_vars: ArgList::from("x"),
        depend_args1: ArgList::from("x"),
        depend_args2: ArgList::from("x.party, x"),
    })))?;

    market.do_request(Request::Create(Item::Depend(Depend {
        depend_type: String::from("implies"),
        depend_pred1: candidate2020.clone(),
        depend_pred2: party2020,
        depend_vars: ArgList::from("x"),
        depend_args1: ArgList::from("x"),
        depend_args2: ArgList::from("x.party"),
    })))?;

    market.do_request(Request::Create(Item::Pred(Pred {
        pred_name: String::from("Atmospheric CO2 levels pass 500ppm"),
        pred_args: ArgList::from("time"),
        pred_value: None,
    })))?;

    let trump_elected = market
        .do_request(Request::Create(Item::Cond(Cond {
            cond_pred: candidate2020.clone(),
            cond_args: vec![trump.clone()],
        })))?
        .unwrap_id();

    let offer_id = market
        .do_request(Request::Create(Item::Offer(Offer {
            offer_user: mrfoo.clone(),
            offer_cond_id: trump_elected.clone(),
            offer_cond_time: None,
            offer_details: OfferDetails {
                offer_buy_price: Dollars::from_millibucks(340),
                offer_sell_price: Dollars::from_millibucks(450),
                offer_buy_quantity: 100,
                offer_sell_quantity: 200,
            },
        })))?
        .unwrap_id();

    market.do_request(Request::Update {
        id: offer_id,
        item_update: ItemUpdate::Offer(OfferDetails {
            offer_buy_price: Dollars::from_millibucks(360),
            offer_sell_price: Dollars::from_millibucks(430),
            offer_buy_quantity: 150,
            offer_sell_quantity: 180,
        }),
    })?;

    let iou_id = market
        .do_request(Request::Create(Item::IOU(IOU {
            iou_issuer: mrfoo.clone(),
            iou_holder: mrbar.clone(),
            iou_value: Dollars::from_millibucks(170),
            iou_cond_id: Some(trump_elected),
            iou_cond_flag: true,
            iou_cond_time: None,
            iou_split: None,
            iou_void: false,
        })))?
        .unwrap_id();
    /*
        market.do_request(Request::Update {
            id: iou_id,
            item_update: ItemUpdate::Void
        })?;
    */
    let mut holders = HashMap::new();
    holders.insert(mrfoo.clone(), Dollars::from_millibucks(120));
    holders.insert(mrbar.clone(), Dollars::from_millibucks(50));
    let transfer = Transfer { holders };

    market.do_request(Request::Update {
        id: iou_id,
        item_update: ItemUpdate::Transfer(transfer),
    })?;

    Ok(())
}

fn status(config: &Config) -> Result<(), Error> {
    let db = DB::open_read_only(&config.db_filename)?;
    let mut market = Market::open_existing(db)?;
    println!("{:?}", market.info);
    market.do_request(Request::Query(Query::AllUser))?.print();
    market.do_request(Request::Query(Query::AllIOU))?.print();
    market.do_request(Request::Query(Query::AllCond))?.print();
    market.do_request(Request::Query(Query::AllOffer))?.print();
    market.do_request(Request::Query(Query::AllEntity))?.print();
    market.do_request(Request::Query(Query::AllRel))?.print();
    market.do_request(Request::Query(Query::AllPred))?.print();
    market.do_request(Request::Query(Query::AllDepend))?.print();
    Ok(())
}

impl Response {
    fn unwrap_id(self) -> ID {
        match self {
            Response::Created(id) => id,
            Response::Updated => panic!("expected ID!"),
            Response::Items(_) => panic!("expected ID!"),
            Response::Error(_) => panic!("expected ID!"),
        }
    }

    fn print(&self) {
        println!("{}", serde_json::to_string(self).unwrap())
    }
}

// vi: ts=8 sts=4 et
