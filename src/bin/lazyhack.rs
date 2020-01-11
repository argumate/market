use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

// measured in cents
type Price = i32;

struct Dollars(Price);

impl Display for Dollars {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let value = self.0.abs();
        let dollars = value / 100;
        let cents = value % 100;
        if self.0 < 0 {
            write!(f, "-${}.{:02}", dollars, cents)
        } else {
            write!(f, "${}.{:02}", dollars, cents)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ContractID(usize);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct PlayerID(usize);

struct Market {
    contract_names: BTreeMap<String, ContractID>,
    player_names: BTreeMap<String, PlayerID>,
    contracts: Vec<Contract>,
    players: Vec<Player>,
    ious: Vec<Iou>,
}

struct Contract {
    name: String,
    outcome: Option<bool>,
}

struct Player {
    name: String,
    ranges: BTreeMap<ContractID, (Price, Price)>,
    credit_limit: Price,
}

struct Exposure {
    unconditional: Price,
    conditional: BTreeMap<ContractID, ContractExposure>,
}

#[derive(Copy, Clone)]
struct ContractExposure {
    exposure: Price,
    neg_exposure: Price,
}

struct Iou {
    pub issuer_id: PlayerID,
    pub holder_id: PlayerID,
    pub status: IouStatus,
    pub amount: Price,
}

#[derive(Eq, PartialEq)]
enum IouStatus {
    Void(Condition),
    True(Condition),
    Unknown(Condition),
}

#[derive(Eq, PartialEq)]
enum Condition {
    If(ContractID),
    Not(ContractID),
}

struct Trade {
    contract_id: ContractID,
    trade_units: i32,
    amount: Price,
}

struct Session {
    exposures: BTreeMap<PlayerID, Exposure>,
    ious: Vec<Iou>,
    trades: Vec<Trade>,
}

struct Offers {
    offers: BTreeMap<ContractID, ContractOffers>,
}

struct ContractOffers {
    buy: BTreeMap<Price, BTreeMap<PlayerID, Price>>,
    sell: BTreeMap<Price, BTreeMap<PlayerID, Price>>,
}

impl Market {
    pub fn new() -> Self {
        let contract_names = BTreeMap::new();
        let player_names = BTreeMap::new();
        let contracts = Vec::new();
        let players = Vec::new();
        let ious = Vec::new();
        Market {
            contract_names,
            player_names,
            contracts,
            players,
            ious,
        }
    }

    pub fn get_contract(&self, contract_id: ContractID) -> &Contract {
        &self.contracts[contract_id.0]
    }

    pub fn get_contract_mut(&mut self, contract_id: ContractID) -> &mut Contract {
        &mut self.contracts[contract_id.0]
    }

    pub fn get_player(&self, player_id: PlayerID) -> &Player {
        &self.players[player_id.0]
    }

    pub fn get_player_mut(&mut self, player_id: PlayerID) -> &mut Player {
        &mut self.players[player_id.0]
    }

    pub fn add_contract(&mut self, contract: Contract) {
        let contract_id = ContractID(self.contracts.len());
        match self
            .contract_names
            .insert(contract.name.clone(), contract_id)
        {
            None => {}
            Some(_old) => panic!("contract already exists: {}", contract.name),
        }
        self.contracts.push(contract);
    }

    pub fn add_player(&mut self, player: Player) {
        let player_id = PlayerID(self.players.len());
        match self.player_names.insert(player.name.clone(), player_id) {
            None => {}
            Some(_old) => panic!("player already exists: {}", player.name),
        }
        self.players.push(player);
    }

    pub fn new_contract(&mut self, name: &str) {
        let contract = Contract::new(name);
        self.add_contract(contract);
    }

    pub fn contract_outcome(&mut self, name: &str, outcome: bool) {
        let contract_id = match self.contract_names.get(name) {
            Some(contract_id) => *contract_id,
            None => panic!("no such contract: {}", name),
        };
        let contract = self.get_contract_mut(contract_id);
        match contract.outcome {
            None => contract.outcome = Some(outcome),
            Some(_outcome) => panic!("contract already has outcome: {}", name),
        }
        for player in &mut self.players {
            player.ranges.remove(&contract_id);
        }
        for iou in &mut self.ious {
            if iou.status == IouStatus::Unknown(Condition::If(contract_id)) {
                if outcome {
                    iou.status = IouStatus::True(Condition::If(contract_id));
                } else {
                    iou.status = IouStatus::Void(Condition::If(contract_id));
                }
            } else if iou.status == IouStatus::Unknown(Condition::Not(contract_id)) {
                if outcome {
                    iou.status = IouStatus::Void(Condition::Not(contract_id));
                } else {
                    iou.status = IouStatus::True(Condition::Not(contract_id));
                }
            }
        }
    }

    pub fn new_player(&mut self, name: &str) {
        let player = Player::new(name);
        self.add_player(player);
    }

    pub fn rename_player(&mut self, old_name: &str, new_name: &str) {
        let player_id = match self.player_names.remove(old_name) {
            Some(player_id) => player_id,
            None => panic!("no such player: {}", old_name),
        };
        match self.player_names.insert(new_name.to_string(), player_id) {
            None => {}
            Some(_old_player_id) => panic!("existing player: {}", new_name),
        }
        self.get_player_mut(player_id).name = new_name.to_string();
    }

    pub fn player_ranges(&mut self, name: &str, ranges: Vec<(&str, Price, Price)>) {
        let player_id = match self.player_names.get(name) {
            Some(player_id) => *player_id,
            None => panic!("no such player: {}", name),
        };
        self.get_player_mut(player_id).clear_ranges();
        for (contract_name, low, high) in ranges {
            let contract_id = match self.contract_names.get(contract_name) {
                Some(contract_id) => *contract_id,
                None => panic!("contract does not exist: {}", contract_name),
            };
            match self.get_contract(contract_id).outcome {
                None => {}
                Some(_outcome) => panic!("contract already has outcome: {}", contract_name),
            }
            if !(0 <= low && low < high && high <= 100) {
                panic!("invalid range: {} {}-{}", contract_name, low, high);
            }
            let player = self.get_player_mut(player_id);
            player.set_range(contract_id, low, high);
        }
    }

    pub fn increment_credit(&mut self, amount: Price) {
        for player in &mut self.players {
            player.credit_limit += amount;
        }
    }

    pub fn dump(&self) {
        println!("CONTRACTS");
        for (contract_name, contract_id) in &self.contract_names {
            if self.get_contract(*contract_id).outcome.is_none() {
                println!(" - {}", contract_name);
            }
        }
        println!();

        println!("PLAYERS ({})", self.players.len());
        for (player_name, _player_id) in &self.player_names {
            println!(" - {}", player_name);
        }
        println!();

        for (player_name, player_id) in &self.player_names {
            println!("{}", player_name);
            let player = self.get_player(*player_id);
            for (contract_name, contract_id) in &self.contract_names {
                if let Some((low, high)) = player.ranges.get(contract_id) {
                    println!(" - {} {}-{}", contract_name, low, high);
                }
            }
            println!();
        }
    }

    pub fn dump_aftermath(&self) {
        println!("IOUs ({})", self.ious.len());
        for iou in &self.ious {
            let issuer_name = &self.get_player(iou.issuer_id).name;
            let holder_name = &self.get_player(iou.holder_id).name;
            match iou.status.decompose() {
                (Condition::If(contract_id), outcome) => println!(
                    " - {} owes {} to {} if {}{}",
                    issuer_name,
                    Dollars(iou.amount),
                    holder_name,
                    self.get_contract(*contract_id).name,
                    match outcome {
                        Some(false) => " -- VOID",
                        Some(true) => " -- TRUE",
                        None => "",
                    }
                ),
                (Condition::Not(contract_id), outcome) => println!(
                    " - {} owes {} to {} if NOT {}{}",
                    issuer_name,
                    Dollars(iou.amount),
                    holder_name,
                    self.get_contract(*contract_id).name,
                    match outcome {
                        Some(false) => " -- VOID",
                        Some(true) => " -- TRUE",
                        None => "",
                    }
                ),
            }
        }
        println!();

        println!("OUTCOMES");
        println!();

        for (player_name, player_id) in &self.player_names {
            println!("{}", player_name);
            let exposure = self.calc_exposure(*player_id);
            let otherwise = exposure.otherwise_outcome();
            let mut outcomes: Vec<(Price, ContractID)> = Vec::new();
            for contract_id in self.contract_names.values() {
                let outcome = exposure.outcome(*contract_id);
                if outcome != 0 && outcome != otherwise {
                    outcomes.push((outcome, *contract_id));
                }
            }
            if !outcomes.is_empty() || otherwise != 0 {
                outcomes.sort();
                outcomes.reverse();
                for (outcome, contract_id) in outcomes.into_iter() {
                    let contract = self.get_contract(contract_id);
                    println!(" - {}: {}", contract.name, Dollars(outcome));
                }
                if otherwise != 0 {
                    println!(" - other: {}", Dollars(otherwise));
                }
            } else {
                println!(" - no outcomes");
            }
            println!();
        }
    }

    pub fn calc_exposure(&self, player_id: PlayerID) -> Exposure {
        let mut exposure = Exposure::new();
        for iou in &self.ious {
            if iou.issuer_id == player_id {
                exposure.apply_iou_issuer(&iou.status, iou.amount);
            } else if iou.holder_id == player_id {
                exposure.apply_iou_holder(&iou.status, iou.amount);
            }
        }
        exposure
    }

    pub fn check_credit_failure(&self, session: &Session, player_id: PlayerID) {
        let player = self.get_player(player_id);
        let exposure = session.exposures.get(&player_id).unwrap();
        for contract_id in self.contract_names.values() {
            let ex = exposure.total_exposure_to_contract(*contract_id);
            if ex > player.credit_limit {
                let contract = self.get_contract(*contract_id);
                panic!(
                    "{}: {} exposed {} > {}",
                    player.name, contract.name, ex, player.credit_limit
                );
            }
            let mut total = 0;
            for iou in self.ious.iter().chain(session.ious.iter()) {
                if iou.issuer_id == player_id {
                    match iou.status {
                        IouStatus::Void(_) => {}
                        IouStatus::True(_) => total += iou.amount,
                        IouStatus::Unknown(Condition::If(contract_id0)) => {
                            if contract_id0 == *contract_id {
                                total += iou.amount;
                            }
                        }
                        IouStatus::Unknown(Condition::Not(contract_id0)) => {
                            if contract_id0 != *contract_id {
                                total += iou.amount;
                            }
                        }
                    }
                } else if iou.holder_id == player_id {
                    match iou.status {
                        IouStatus::Void(_) => {}
                        IouStatus::True(_) => total -= iou.amount,
                        IouStatus::Unknown(Condition::If(contract_id0)) => {
                            if contract_id0 == *contract_id {
                                total -= iou.amount;
                            }
                        }
                        IouStatus::Unknown(Condition::Not(contract_id0)) => {
                            if contract_id0 != *contract_id {
                                total -= iou.amount;
                            }
                        }
                    }
                }
            }
            if ex != total {
                panic!("exposure does not match: {} vs. {}", ex, total);
            }
        }
    }

    pub fn player_max_buy_amount(
        &self,
        session: &Session,
        buyer_id: PlayerID,
        contract_id: ContractID,
    ) -> Price {
        let buyer_credit_limit = self.get_player(buyer_id).credit_limit;
        let buyer_exposure = session.exposures.get(&buyer_id).unwrap();
        let buyer_exposed = buyer_exposure.total_exposure_to_contract_neg(contract_id);
        let buyer_max_amount = max(0, buyer_credit_limit - buyer_exposed);
        buyer_max_amount
    }

    pub fn player_max_sell_amount(
        &self,
        session: &Session,
        seller_id: PlayerID,
        contract_id: ContractID,
    ) -> Price {
        let seller_credit_limit = self.get_player(seller_id).credit_limit;
        let seller_exposure = session.exposures.get(&seller_id).unwrap();
        let seller_exposed = seller_exposure.total_exposure_to_contract(contract_id);
        let seller_max_amount = max(0, seller_credit_limit - seller_exposed);
        seller_max_amount
    }

    pub fn session(&mut self) {
        let mut session = Session::new(self);

        loop {
            let offers = session.find_offers(self);
            let trades = session.find_trades(self, &offers);

            if let Some((_spread, _contract_name, contract_id, buyer_id, high, seller_id, low)) =
                trades.into_iter().next()
            {
                let price = (low + high) / 2;

                //println!("trade: {}", self.get_contract(contract_id).name);
                //println!("price: {}", price);
                //println!();

                let buyer_max_amount = self.player_max_buy_amount(&session, buyer_id, contract_id);
                let buyer_max_units = buyer_max_amount / price;

                //println!("buyer: {}", self.get_player(buyer_id).name);
                //println!("buyer_max_amount = {}", buyer_max_amount);
                //println!("buyer_max_units = {}", buyer_max_units);
                //println!();

                let seller_max_amount =
                    self.player_max_sell_amount(&session, seller_id, contract_id);
                let seller_max_units = seller_max_amount / (100 - price);

                //println!("seller: {}", self.get_player(seller_id).name);
                //println!("seller_max_amount = {}", seller_max_amount);
                //println!("seller_max_units = {}", seller_max_units);
                //println!();

                let trade_units = min(buyer_max_units, seller_max_units);

                if !(trade_units > 0) {
                    panic!("trade_units = {}", trade_units);
                }

                let seller_iou = Iou {
                    issuer_id: seller_id,
                    holder_id: buyer_id,
                    status: IouStatus::Unknown(Condition::If(contract_id)),
                    amount: trade_units * (100 - price),
                };

                let buyer_iou = Iou {
                    issuer_id: buyer_id,
                    holder_id: seller_id,
                    status: IouStatus::Unknown(Condition::Not(contract_id)),
                    amount: trade_units * price,
                };

                /*
                println!(
                    "{} {} units @ {} : {} -> {}",
                    self.get_contract(contract_id).name,
                    trade_units,
                    price,
                    self.get_player(seller_id).name,
                    self.get_player(buyer_id).name
                );
                println!();
                */

                let amount = trade_units * price;
                let trade = Trade {
                    contract_id,
                    trade_units,
                    amount,
                };
                session.trades.push(trade);

                session.push_iou(seller_iou);
                session.push_iou(buyer_iou);

                self.check_credit_failure(&session, buyer_id);
                self.check_credit_failure(&session, seller_id);
            } else {
                //println!("no trades!");
                //println!();
                break;
            }
        }

        let offers = session.find_offers(self);
        let spreads = offers.find_spreads(self);
        println!("SPREADS ({})", spreads.len());
        for (contract_name, (low, high)) in spreads.into_iter() {
            println!(" - {} {}-{}", contract_name, low, high);
        }
        println!();

        self.ious.append(&mut session.ious);

        println!("CLEARED SPREADS");
        for (contract_name, contract_id) in &self.contract_names {
            let mut value = 0;
            let mut price = 0;
            let mut neg_value = 0;
            let mut neg_price = 0;
            for iou in &self.ious {
                if iou.status == IouStatus::Unknown(Condition::If(*contract_id)) {
                    let holder = self.get_player(iou.holder_id);
                    let sell = if let Some((_buy, sell)) = holder.ranges.get(contract_id) {
                        *sell
                    } else {
                        100
                    };
                    value += iou.amount;
                    price += iou.amount * sell / 100;
                } else if iou.status == IouStatus::Unknown(Condition::Not(*contract_id)) {
                    let holder = self.get_player(iou.holder_id);
                    let buy = if let Some((buy, _sell)) = holder.ranges.get(contract_id) {
                        *buy
                    } else {
                        0
                    };
                    neg_value += iou.amount;
                    neg_price += iou.amount * buy / 100;
                }
            }
            if value != 0 && neg_value != 0 {
                let low = neg_price * 100 / neg_value;
                let high = price * 100 / value;
                println!(" - {} {}-{}", contract_name, low, high);
            }
        }
        println!();

        println!("PRICES");
        for (contract_name, contract_id) in &self.contract_names {
            let mut total_units = 0;
            let mut total_amount = 0;
            for trade in &session.trades {
                if trade.contract_id == *contract_id {
                    total_units += trade.trade_units;
                    total_amount += trade.amount;
                }
            }
            if total_amount > 0 {
                let price = total_amount / total_units;
                println!(" - {} {}", contract_name, price);
            }
        }
        println!();
    }
}

impl Session {
    pub fn new(market: &Market) -> Self {
        let mut exposures = BTreeMap::new();
        let ious = Vec::new();
        let trades = Vec::new();
        for (_player_name, player_id) in &market.player_names {
            let exposure = Exposure::new();
            exposures.insert(*player_id, exposure);
        }
        let mut session = Session {
            exposures,
            ious,
            trades,
        };
        for iou in &market.ious {
            session.apply_iou(iou);
        }
        session
    }

    fn apply_iou(&mut self, iou: &Iou) {
        self.exposures
            .get_mut(&iou.issuer_id)
            .unwrap()
            .apply_iou_issuer(&iou.status, iou.amount);
        self.exposures
            .get_mut(&iou.holder_id)
            .unwrap()
            .apply_iou_holder(&iou.status, iou.amount);
    }

    pub fn push_iou(&mut self, iou: Iou) {
        self.apply_iou(&iou);
        self.ious.push(iou);
    }

    pub fn find_offers(&self, market: &Market) -> Offers {
        let mut offers = Offers::new();
        for (_player_name, player_id) in &market.player_names {
            let player = market.get_player(*player_id);
            for (contract_id, (low, high)) in &player.ranges {
                let buy_amount = market.player_max_buy_amount(&self, *player_id, *contract_id);
                let sell_amount = market.player_max_sell_amount(&self, *player_id, *contract_id);
                if buy_amount >= 100 {
                    offers.add_buy_offer(*contract_id, *player_id, *low, buy_amount);
                }
                if sell_amount >= 100 {
                    offers.add_sell_offer(*contract_id, *player_id, *high, sell_amount);
                }
            }
        }
        offers
    }

    pub fn find_trades(
        &self,
        market: &Market,
        offers: &Offers,
    ) -> Vec<(Price, String, ContractID, PlayerID, Price, PlayerID, Price)> {
        let mut trades = Vec::new();
        for (contract_id, contract_offers) in &offers.offers {
            let contract = market.get_contract(*contract_id);
            let mut buy_iter = contract_offers.buy.iter().rev();
            let mut sell_iter = contract_offers.sell.iter();
            if let Some((high, buyers)) = buy_iter.next() {
                if let Some((low, sellers)) = sell_iter.next() {
                    if low <= high {
                        let mut low = low;
                        let mut high = high;
                        match (buy_iter.next(), sell_iter.next()) {
                            (Some((next_high, _next_buyers)), Some((next_low, _next_sellers))) => {
                                if next_low < next_high {
                                    low = next_low;
                                    high = next_high;
                                    //println!("low={} next_low={} next_high={} high={}", low, next_low, next_high, high);
                                }
                            }
                            (Some((next_high, _next_buyers)), None) => {
                                if next_high > low {
                                    high = next_high;
                                    //println!("low={} next_high={} high={}", low, next_high, high);
                                }
                            }
                            (None, Some((next_low, _next_sellers))) => {
                                if next_low < high {
                                    low = next_low;
                                    //println!("low={} next_low={} high={}", low, next_low, high);
                                }
                            }
                            (None, None) => {}
                        }
                        let mut buyers: Vec<(Price, String, PlayerID)> = buyers
                            .iter()
                            .map(|(buyer_id, amount)| {
                                (
                                    *amount,
                                    market.get_player(*buyer_id).name.clone(),
                                    *buyer_id,
                                )
                            })
                            .collect();
                        let mut sellers: Vec<(Price, String, PlayerID)> = sellers
                            .iter()
                            .map(|(seller_id, amount)| {
                                (
                                    *amount,
                                    market.get_player(*seller_id).name.clone(),
                                    *seller_id,
                                )
                            })
                            .collect();

                        // sort by amount, then name
                        // then remove all that aren't the highest amount

                        buyers.sort();
                        let max_buy_amount = buyers[buyers.len() - 1].0;
                        buyers.retain(|(amount, _name, _id)| *amount == max_buy_amount);

                        sellers.sort();
                        let max_sell_amount = sellers[sellers.len() - 1].0;
                        sellers.retain(|(amount, _name, _id)| *amount == max_sell_amount);

                        /*
                        if buyers.len() > 1 {
                            println!(
                                "multiple buyers for {} at {}: {:?}",
                                contract.name, high, buyers
                            );
                        }

                        if sellers.len() > 1 {
                            println!(
                                "multiple sellers for {} at {}: {:?}",
                                contract.name, low, sellers
                            );
                        }
                        */

                        let (_buyer_amount, _buyer_name, buyer_id) =
                            buyers.into_iter().next().unwrap(); // FIXME
                        let (_seller_amount, _seller_name, seller_id) =
                            sellers.into_iter().next().unwrap(); // FIXME
                        let spread = high - low;
                        trades.push((
                            spread,
                            contract.name.clone(), // for consistent ordering
                            *contract_id,
                            buyer_id,
                            *high,
                            seller_id,
                            *low,
                        ));
                    }
                }
            }
        }
        trades.sort();
        trades.reverse();
        trades
    }
}

impl Offers {
    pub fn new() -> Self {
        let offers = BTreeMap::new();
        Offers { offers }
    }

    pub fn add_buy_offer(
        &mut self,
        contract_id: ContractID,
        player_id: PlayerID,
        low: Price,
        amount: Price,
    ) {
        self.offers
            .entry(contract_id)
            .or_insert_with(|| ContractOffers::new())
            .buy
            .entry(low)
            .or_insert_with(|| BTreeMap::new())
            .insert(player_id, amount);
    }

    pub fn add_sell_offer(
        &mut self,
        contract_id: ContractID,
        player_id: PlayerID,
        high: Price,
        amount: Price,
    ) {
        self.offers
            .entry(contract_id)
            .or_insert_with(|| ContractOffers::new())
            .sell
            .entry(high)
            .or_insert_with(|| BTreeMap::new())
            .insert(player_id, amount);
    }

    pub fn find_spreads(&self, market: &Market) -> BTreeMap<String, (Price, Price)> {
        let mut spreads = BTreeMap::new();
        for (contract_id, offers) in &self.offers {
            let contract = market.get_contract(*contract_id);
            if contract.outcome == None {
                let buy = *offers.buy.keys().rev().next().unwrap_or(&0);
                let sell = *offers.sell.keys().next().unwrap_or(&100);
                if sell <= buy {
                    panic!("you said no trades! {} {} {}", contract.name, buy, sell);
                }
                spreads.insert(contract.name.clone(), (buy, sell));
            }
        }
        spreads
    }
}

impl ContractOffers {
    pub fn new() -> Self {
        let buy = BTreeMap::new();
        let sell = BTreeMap::new();
        ContractOffers { buy, sell }
    }
}

impl Contract {
    pub fn new(name: impl Into<String>) -> Self {
        let outcome = None;
        Contract {
            name: name.into(),
            outcome,
        }
    }
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        let ranges = BTreeMap::new();
        let credit_limit = 0;
        Player {
            name: name.into(),
            ranges,
            credit_limit,
        }
    }

    pub fn clear_ranges(&mut self) {
        self.ranges.clear();
    }

    pub fn set_range(&mut self, contract_id: ContractID, low: Price, high: Price) {
        self.ranges.insert(contract_id, (low, high));
    }
}

impl Exposure {
    pub fn new() -> Self {
        let unconditional = 0;
        let conditional = BTreeMap::new();
        Exposure {
            unconditional,
            conditional,
        }
    }

    pub fn apply_iou_issuer(&mut self, status: &IouStatus, amount: Price) {
        match status {
            IouStatus::Void(_) => {}
            IouStatus::True(_) => self.unconditional += amount,
            IouStatus::Unknown(Condition::If(contract_id)) => {
                self.add_exposure(*contract_id, amount)
            }
            IouStatus::Unknown(Condition::Not(contract_id)) => {
                self.add_neg_exposure(*contract_id, amount)
            }
        }
    }

    pub fn apply_iou_holder(&mut self, status: &IouStatus, amount: Price) {
        match status {
            IouStatus::Void(_) => {}
            IouStatus::True(_) => self.unconditional -= amount,
            IouStatus::Unknown(Condition::If(contract_id)) => {
                self.add_exposure(*contract_id, -amount)
            }
            IouStatus::Unknown(Condition::Not(contract_id)) => {
                self.add_neg_exposure(*contract_id, -amount)
            }
        }
    }

    fn contract_exposure(&self, contract_id: ContractID) -> ContractExposure {
        self.conditional
            .get(&contract_id)
            .map(|x| *x)
            .unwrap_or(ContractExposure::zero())
    }

    fn exposure(&self, contract_id: ContractID) -> Price {
        self.contract_exposure(contract_id).exposure
    }

    fn neg_exposure(&self, contract_id: ContractID) -> Price {
        self.contract_exposure(contract_id).neg_exposure
    }

    // exposure to P:
    // - how much debt minus assets we owe conditional on P
    // - plus how much debt minus assets we owe conditional on ~Q, where Q \= P
    // - plus unconditional debt
    pub fn total_exposure_to_contract(&self, contract_id: ContractID) -> Price {
        self.exposure(contract_id) + self.total_neg_exposure() - self.neg_exposure(contract_id)
    }

    // exposure to ~P:
    // - how much debt minus assets we owe conditional on ~Q for all Q
    // - plus biggest [ debt minus assets we owe conditional on Q where Q \= P
    //     minus the debt minus assets we owe conditional on ~Q ] if positive
    // - plus unconditional debt
    pub fn total_exposure_to_contract_neg(&self, contract_id: ContractID) -> Price {
        self.total_neg_exposure()
            + max(
                0,
                self.conditional
                    .iter()
                    .filter(|(contract_id0, _contract_exposure)| **contract_id0 != contract_id)
                    .map(|(_contract_id0, contract_exposure)| {
                        contract_exposure.exposure - contract_exposure.neg_exposure
                    })
                    .max()
                    .unwrap_or(0),
            )
    }

    // total exposure to ~Q for all Q:
    // - how much debt minus assets we owe conditional on ~Q for all Q
    // - plus unconditional debt
    pub fn total_neg_exposure(&self) -> Price {
        let neg: Price = self.conditional.values().map(|x| x.neg_exposure).sum();
        neg + self.unconditional
    }

    pub fn outcome(&self, contract_id: ContractID) -> Price {
        -self.total_exposure_to_contract(contract_id)
    }

    pub fn otherwise_outcome(&self) -> Price {
        -self.total_neg_exposure()
    }

    pub fn add_exposure(&mut self, contract_id: ContractID, amount: Price) {
        self.conditional
            .entry(contract_id)
            .and_modify(|contract_exposure| contract_exposure.exposure += amount)
            .or_insert(ContractExposure::exposure(amount));
    }

    pub fn add_neg_exposure(&mut self, contract_id: ContractID, amount: Price) {
        self.conditional
            .entry(contract_id)
            .and_modify(|contract_exposure| contract_exposure.neg_exposure += amount)
            .or_insert(ContractExposure::neg_exposure(amount));
    }
}

impl ContractExposure {
    pub fn zero() -> Self {
        ContractExposure {
            exposure: 0,
            neg_exposure: 0,
        }
    }

    pub fn exposure(exposure: Price) -> Self {
        ContractExposure {
            exposure,
            neg_exposure: 0,
        }
    }

    pub fn neg_exposure(neg_exposure: Price) -> Self {
        ContractExposure {
            exposure: 0,
            neg_exposure,
        }
    }
}

impl IouStatus {
    pub fn decompose(&self) -> (&Condition, Option<bool>) {
        match self {
            IouStatus::Void(condition) => (condition, Some(false)),
            IouStatus::True(condition) => (condition, Some(true)),
            IouStatus::Unknown(condition) => (condition, None),
        }
    }
}

fn main() {
    let mut market = Market::new();

    // Sun 10 Nov 2019
    setup_session1(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 16 Nov 2019
    setup_session2(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 23 Nov 2019
    setup_session3(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 30 Nov 2019
    setup_session4(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 07 Dec 2019
    setup_session5(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 14 Dec 2019
    setup_session6(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 21 Dec 2019
    setup_session7(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 28 Dec 2019
    setup_session8(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 04 Jan 2020
    setup_session9(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 11 Jan 2020
    setup_session10(&mut market);
    market.increment_credit(1000);

    market.dump();
    market.session();
    market.dump_aftermath();
}

fn setup_session1(market: &mut Market) {
    market.new_contract("Sanders");
    market.new_contract("Warren");
    market.new_contract("Biden");
    market.new_contract("Buttigieg");
    market.new_contract("Harris");
    market.new_contract("Steyer");
    market.new_contract("Yang");
    market.new_contract("Gillibrand");
    market.new_contract("Gabbard");
    market.new_contract("Williamson");
    market.new_contract("Booker");
    market.new_contract("Klobuchar");
    market.new_contract("Castro");

    market.new_player("antoine-roquentin");
    market.new_player("blastfarmer");
    market.new_player("cromulentenough");
    market.new_player("flakmaniak");
    market.new_player("intercal");
    market.new_player("kwarrtz");
    market.new_player("learn-tilde-ath");
    market.new_player("ouroborostriumphant");
    market.new_player("placid-platypus");
    market.new_player("squareallworthy");
    market.new_player("tanadrin");
    market.new_player("the-moti");
    market.new_player("triviallytrue");
    market.new_player("von-hresvelg");

    market.player_ranges("learn-tilde-ath", vec![("Sanders", 2, 98)]);

    market.player_ranges(
        "antoine-roquentin",
        vec![
            ("Sanders", 79, 80),
            ("Warren", 14, 15),
            ("Biden", 0, 1),
            ("Buttigieg", 0, 1),
            ("Harris", 0, 1),
            ("Steyer", 0, 1),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "triviallytrue",
        vec![
            ("Sanders", 20, 40),
            ("Warren", 25, 45),
            ("Buttigieg", 15, 30),
        ],
    );

    market.player_ranges(
        "ouroborostriumphant",
        vec![
            ("Biden", 20, 45),
            ("Warren", 20, 45),
            ("Sanders", 10, 20),
            ("Yang", 5, 10),
            ("Buttigieg", 5, 15),
            ("Harris", 5, 10),
            ("Gillibrand", 0, 5),
            ("Gabbard", 0, 5),
            ("Williamson", 0, 5),
        ],
    );

    market.player_ranges(
        "tanadrin",
        vec![("Warren", 20, 40), ("Biden", 20, 40), ("Sanders", 10, 20)],
    );

    market.player_ranges(
        "kwarrtz",
        vec![
            ("Warren", 40, 60),
            ("Biden", 40, 60),
            ("Sanders", 20, 25),
            ("Buttigieg", 0, 10),
            ("Harris", 0, 5),
            ("Yang", 0, 1),
            ("Steyer", 0, 1),
        ],
    );

    market.player_ranges(
        "flakmaniak",
        vec![
            ("Warren", 30, 60),
            ("Biden", 25, 45),
            ("Sanders", 10, 30),
            ("Buttigieg", 0, 8),
            ("Harris", 0, 6),
            ("Booker", 0, 4),
        ],
    );

    market.player_ranges(
        "squareallworthy",
        vec![
            ("Biden", 39, 41),
            ("Warren", 32, 34),
            ("Sanders", 9, 11),
            ("Buttigieg", 4, 6),
            ("Harris", 4, 6),
            ("Yang", 0, 1),
            ("Gabbard", 0, 1),
            ("Booker", 0, 1),
            ("Klobuchar", 0, 1),
        ],
    );

    market.player_ranges(
        "intercal",
        vec![
            ("Warren", 15, 25),
            ("Biden", 50, 75),
            ("Sanders", 0, 15),
            ("Yang", 0, 5),
        ],
    );

    market.player_ranges(
        "cromulentenough",
        vec![
            ("Warren", 25, 40),
            ("Sanders", 20, 35),
            ("Biden", 10, 25),
            ("Buttigieg", 5, 15),
        ],
    );

    market.player_ranges(
        "von-hresvelg",
        vec![
            ("Warren", 40, 60),
            ("Biden", 35, 50),
            ("Sanders", 20, 35),
            ("Buttigieg", 5, 25),
            ("Yang", 0, 15),
        ],
    );

    market.player_ranges(
        "placid-platypus",
        vec![
            ("Biden", 15, 35),
            ("Warren", 20, 40),
            ("Sanders", 10, 20),
            ("Buttigieg", 5, 15),
            ("Yang", 2, 8),
            ("Harris", 2, 6),
            ("Booker", 0, 5),
            ("Castro", 0, 4),
        ],
    );

    market.player_ranges(
        "blastfarmer",
        vec![
            ("Biden", 0, 33),
            ("Warren", 0, 46),
            ("Sanders", 0, 11),
            ("Buttigieg", 0, 9),
        ],
    );

    market.player_ranges(
        "the-moti",
        vec![
            ("Warren", 23, 46),
            ("Biden", 15, 30),
            ("Buttigieg", 13, 26),
            ("Sanders", 9, 18),
            ("Yang", 5, 10),
            ("Klobuchar", 2, 4),
        ],
    );
}

fn setup_session2(market: &mut Market) {
    market.new_contract("Patrick");

    market.new_player("birth-muffins-death");
    market.new_player("bibliolithid");
    market.new_player("firebendinglemur");
    market.new_player("tremorbond");
    market.new_player("la-pou-belle");

    market.rename_player("blastfarmer", "irradiate-space");

    market.player_ranges(
        "cromulentenough",
        vec![
            ("Sanders", 30, 50),
            ("Warren", 30, 50),
            ("Biden", 15, 35),
            ("Buttigieg", 20, 40),
        ],
    );

    market.player_ranges(
        "flakmaniak",
        vec![
            ("Warren", 30, 60),
            ("Biden", 25, 45),
            ("Sanders", 10, 30),
            ("Buttigieg", 0, 8),
            ("Harris", 0, 6),
            ("Booker", 0, 4),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "birth-muffins-death",
        vec![
            ("Biden", 35, 75),
            ("Warren", 10, 30),
            ("Sanders", 5, 35),
            ("Buttigieg", 1, 10),
            ("Harris", 1, 3),
        ],
    );

    market.player_ranges(
        "ouroborostriumphant",
        vec![
            ("Biden", 20, 45),
            ("Buttigieg", 5, 15),
            ("Gabbard", 0, 5),
            ("Gillibrand", 0, 5),
            ("Harris", 2, 10),
            ("Sanders", 10, 20),
            ("Warren", 20, 45),
            ("Williamson", 0, 5),
            ("Yang", 1, 7),
        ],
    );

    market.player_ranges(
        "bibliolithid",
        vec![
            ("Warren", 40, 45),
            ("Biden", 28, 32),
            ("Sanders", 17, 21),
            ("Buttigieg", 3, 5),
            ("Yang", 1, 2),
            ("Harris", 1, 3),
        ],
    );

    market.player_ranges(
        "irradiate-space",
        vec![
            ("Biden", 20, 33),
            ("Warren", 20, 47),
            ("Sanders", 0, 11),
            ("Buttigieg", 0, 9),
            ("Yang", 0, 1),
            ("Harris", 0, 1),
        ],
    );

    market.player_ranges(
        "squareallworthy",
        vec![
            ("Biden", 40, 41),
            ("Booker", 0, 3),
            ("Buttigieg", 7, 8),
            ("Gabbard", 0, 3),
            ("Harris", 3, 4),
            ("Klobuchar", 0, 3),
            ("Sanders", 9, 10),
            ("Warren", 34, 35),
            ("Yang", 0, 3),
            ("Patrick", 0, 3),
        ],
    );

    market.player_ranges(
        "tanadrin",
        vec![
            ("Biden", 20, 40),
            ("Sanders", 10, 20),
            ("Warren", 20, 40),
            ("Yang", 0, 1),
            ("Williamson", 0, 1),
            ("Steyer", 0, 1),
            ("Klobuchar", 0, 1),
            ("Gillibrand", 0, 1),
            ("Gabbard", 0, 1),
            ("Castro", 0, 1),
            ("Buttigieg", 0, 1),
            ("Booker", 0, 1),
            ("Harris", 0, 10),
        ],
    );

    market.player_ranges(
        "the-moti",
        vec![
            ("Booker", 0, 1),
            ("Castro", 0, 2),
            ("Gabbard", 2, 6),
            ("Gillibrand", 0, 2),
            ("Harris", 2, 6),
            ("Klobuchar", 1, 3),
            ("Steyer", 0, 1),
            ("Williamson", 0, 2),
            ("Yang", 4, 8),
            ("Biden", 20, 28),
            ("Buttigieg", 14, 22),
            ("Sanders", 9, 17),
            ("Warren", 24, 32),
        ],
    );

    market.player_ranges(
        "firebendinglemur",
        vec![
            ("Biden", 65, 70),
            ("Warren", 15, 17),
            ("Sanders", 11, 12),
            ("Buttigieg", 2, 3),
            ("Harris", 1, 2),
            ("Gabbard", 0, 2),
        ],
    );

    market.player_ranges(
        "tremorbond",
        vec![
            ("Biden", 33, 39),
            ("Warren", 29, 35),
            ("Sanders", 13, 17),
            ("Buttigieg", 8, 12),
            ("Harris", 1, 3),
            ("Yang", 1, 3),
        ],
    );

    market.player_ranges(
        "la-pou-belle",
        vec![
            ("Warren", 54, 56),
            ("Biden", 24, 26),
            ("Buttigieg", 11, 13),
            ("Sanders", 5, 7),
            ("Steyer", 1, 3),
        ],
    );
}

fn setup_session3(market: &mut Market) {
    market.new_contract("Bloomberg");

    market.new_player("maybesimon");

    market.player_ranges(
        "squareallworthy",
        vec![
            ("Biden", 42, 43),
            ("Buttigieg", 7, 9),
            ("Harris", 2, 3),
            ("Sanders", 9, 11),
            ("Warren", 34, 35),
            ("Yang", 0, 1),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 1),
            ("Patrick", 0, 1),
        ],
    );

    market.player_ranges(
        "placid-platypus",
        vec![
            ("Biden", 20, 35),
            ("Warren", 18, 28),
            ("Sanders", 10, 15),
            ("Buttigieg", 10, 18),
            ("Yang", 1, 6),
            ("Harris", 2, 5),
            ("Klobuchar", 3, 6),
            ("Booker", 0, 1),
            ("Castro", 0, 1),
            ("Patrick", 0, 2),
            ("Bloomberg", 0, 1),
        ],
    );

    market.player_ranges(
        "kwarrtz",
        vec![
            ("Warren", 40, 60),
            ("Biden", 40, 60),
            ("Sanders", 20, 25),
            ("Buttigieg", 0, 10),
            ("Harris", 0, 5),
            ("Yang", 0, 1),
            ("Steyer", 0, 1),
            ("Patrick", 0, 5),
            ("Bloomberg", 0, 10),
            ("Klobuchar", 0, 1),
        ],
    );

    market.player_ranges(
        "tanadrin",
        vec![
            ("Biden", 20, 40),
            ("Sanders", 10, 20),
            ("Warren", 20, 40),
            ("Yang", 0, 1),
            ("Williamson", 0, 1),
            ("Steyer", 0, 1),
            ("Klobuchar", 0, 1),
            ("Gillibrand", 0, 1),
            ("Gabbard", 0, 1),
            ("Castro", 0, 1),
            ("Buttigieg", 5, 15),
            ("Booker", 0, 1),
            ("Harris", 0, 10),
        ],
    );

    market.player_ranges(
        "the-moti",
        vec![
            ("Biden", 21, 23),
            ("Bloomberg", 2, 8),
            ("Booker", 0, 2),
            ("Buttigieg", 16, 23),
            ("Harris", 2, 4),
            ("Klobuchar", 2, 4),
            ("Sanders", 12, 16),
            ("Warren", 22, 25),
            ("Yang", 3, 8),
        ],
    );

    market.player_ranges("maybesimon", vec![("Biden", 75, 95), ("Warren", 25, 30)]);
}

fn setup_session4(market: &mut Market) {
    market.new_contract("Bennet");
    market.new_contract("Bullock");
    market.new_contract("Delaney");
    market.new_contract("Sestak");

    market.new_player("utilitymonstermash");

    market.player_ranges(
        "ouroborostriumphant",
        vec![
            ("Biden", 30, 35),
            ("Warren", 20, 25),
            ("Buttigieg", 10, 20),
            ("Sanders", 10, 15),
            ("Buttigieg", 10, 20),
            ("Harris", 2, 5),
            ("Yang", 1, 5),
            ("Bloomberg", 0, 1),
            ("Patrick", 0, 1),
            ("Gabbard", 0, 1),
            ("Gillibrand", 0, 1),
            ("Klobuchar", 0, 1),
        ],
    );

    market.player_ranges(
        "von-hresvelg",
        vec![
            ("Biden", 40, 70),
            ("Buttigieg", 5, 15),
            ("Sanders", 20, 35),
            ("Warren", 35, 50),
            ("Yang", 0, 5),
        ],
    );

    market.player_ranges(
        "kwarrtz",
        vec![
            ("Warren", 40, 60),
            ("Biden", 40, 60),
            ("Sanders", 20, 25),
            ("Buttigieg", 0, 20),
            ("Harris", 0, 5),
            ("Yang", 0, 1),
            ("Steyer", 0, 1),
            ("Patrick", 0, 5),
            ("Bloomberg", 0, 10),
            ("Klobuchar", 0, 1),
        ],
    );

    market.player_ranges(
        "utilitymonstermash",
        vec![
            ("Biden", 22, 42),
            ("Bloomberg", 1, 5),
            ("Booker", 0, 1),
            ("Buttigieg", 1, 9),
            ("Castro", 0, 1),
            ("Gabbard", 0, 1),
            ("Gillibrand", 0, 1),
            ("Harris", 4, 8),
            ("Klobuchar", 1, 2),
            ("Patrick", 0, 1),
            ("Sanders", 6, 13),
            ("Steyer", 0, 1),
            ("Warren", 22, 33),
            ("Williamson", 0, 1),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "irradiate-space",
        vec![
            ("Biden", 5, 11),
            ("Bloomberg", 5, 11),
            ("Buttigieg", 5, 11),
            ("Harris", 0, 1),
            ("Sanders", 20, 31),
            ("Warren", 20, 47),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "squareallworthy",
        vec![
            ("Biden", 44, 45),
            ("Buttigieg", 5, 10),
            ("Harris", 1, 2),
            ("Sanders", 8, 12),
            ("Warren", 35, 36),
            ("Yang", 0, 1),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 1),
            ("Patrick", 0, 1),
        ],
    );

    market.player_ranges(
        "cromulentenough",
        vec![
            ("Sanders", 25, 35),
            ("Warren", 30, 40),
            ("Biden", 15, 25),
            ("Buttigieg", 15, 30),
            ("Harris", 5, 10),
            ("Gabbard", 1, 4),
            ("Bloomberg", 1, 4),
            ("Yang", 1, 4),
            ("Klobuchar", 0, 2),
            ("Bennet", 0, 2),
            ("Booker", 0, 2),
            ("Williamson", 0, 2),
            ("Patrick", 0, 2),
            ("Sestak", 0, 2),
            ("Steyer", 0, 2),
            ("Delaney", 0, 2),
            ("Castro", 0, 2),
            ("Bullock", 0, 2),
        ],
    );

    market.player_ranges(
        "birth-muffins-death",
        vec![
            ("Biden", 35, 75),
            ("Warren", 10, 30),
            ("Sanders", 5, 35),
            ("Buttigieg", 1, 10),
            ("Harris", 0, 1),
            ("Yang", 0, 1),
        ],
    );
}

fn setup_session5(market: &mut Market) {
    market.contract_outcome("Harris", false);
    market.contract_outcome("Bullock", false);
    market.contract_outcome("Sestak", false);
    market.contract_outcome("Gillibrand", false);

    market.new_player("worriedaboutmyfern");
    market.new_player("ijime-deactivated20150440");

    market.player_ranges(
        "placid-platypus",
        vec![
            ("Biden", 30, 45),
            ("Warren", 25, 35),
            ("Sanders", 12, 16),
            ("Buttigieg", 12, 20),
            ("Yang", 1, 6),
            ("Klobuchar", 1, 4),
            ("Booker", 0, 1),
            ("Castro", 0, 1),
            ("Patrick", 0, 2),
            ("Bloomberg", 0, 1),
            ("Gabbard", 0, 1),
            ("Williamson", 0, 1),
        ],
    );

    market.player_ranges(
        "squareallworthy",
        vec![
            ("Biden", 49, 50),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Buttigieg", 5, 11),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 1),
            ("Patrick", 0, 1),
            ("Sanders", 8, 12),
            ("Warren", 35, 36),
            ("Yang", 0, 1),
            ("Williamson", 0, 1),
        ],
    );

    market.player_ranges(
        "the-moti",
        vec![
            ("Biden", 22, 27),
            ("Buttigieg", 18, 21),
            ("Warren", 16, 18),
            ("Sanders", 12, 17),
            ("Bloomberg", 4, 13),
            ("Yang", 3, 8),
            ("Klobuchar", 1, 4),
            ("Gabbard", 0, 2),
            ("Booker", 0, 1),
        ],
    );

    market.player_ranges(
        "irradiate-space",
        vec![
            ("Biden", 5, 11),
            ("Bloomberg", 5, 11),
            ("Buttigieg", 5, 11),
            ("Sanders", 20, 31),
            ("Steyer", 2, 5),
            ("Warren", 25, 57),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "worriedaboutmyfern",
        vec![
            ("Biden", 55, 80),
            ("Warren", 10, 20),
            ("Buttigieg", 10, 20),
            ("Sanders", 0, 5),
        ],
    );

    market.player_ranges(
        "ijime-deactivated20150440",
        vec![
            ("Biden", 10, 15),
            ("Sanders", 10, 20),
            ("Warren", 10, 40),
            ("Klobuchar", 0, 1),
            ("Buttigieg", 10, 15),
            ("Gabbard", 3, 10),
            ("Booker", 2, 5),
            ("Williamson", 2, 5),
            ("Yang", 3, 25),
        ],
    );

    market.player_ranges(
        "utilitymonstermash",
        vec![
            ("Bennet", 0, 1),
            ("Biden", 21, 45),
            ("Bloomberg", 4, 12),
            ("Booker", 0, 1),
            ("Buttigieg", 18, 33),
            ("Castro", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 1),
            ("Patrick", 0, 1),
            ("Sanders", 4, 10),
            ("Steyer", 0, 1),
            ("Warren", 21, 45),
            ("Williamson", 0, 1),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "kwarrtz",
        vec![
            ("Warren", 40, 60),
            ("Biden", 40, 60),
            ("Sanders", 10, 20),
            ("Buttigieg", 5, 25),
            ("Yang", 0, 1),
            ("Steyer", 0, 1),
            ("Patrick", 0, 5),
            ("Bloomberg", 0, 10),
            ("Klobuchar", 0, 1),
        ],
    );
}

fn setup_session6(market: &mut Market) {
    market.player_ranges(
        "ouroborostriumphant",
        vec![
            ("Biden", 35, 40),
            ("Warren", 20, 25),
            ("Sanders", 15, 20),
            ("Buttigieg", 15, 20),
            ("Yang", 1, 4),
            ("Patrick", 0, 2),
            ("Bloomberg", 0, 2),
            ("Steyer", 0, 1),
            ("Klobuchar", 0, 1),
            ("Bennet", 0, 1),
            ("Booker", 0, 1),
            ("Gabbard", 0, 1),
            ("Williamson", 0, 1),
        ],
    );

    market.player_ranges(
        "utilitymonstermash",
        vec![
            ("Bennet", 0, 1),
            ("Biden", 27, 60),
            ("Bloomberg", 4, 12),
            ("Booker", 0, 1),
            ("Buttigieg", 10, 27),
            ("Castro", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 2),
            ("Patrick", 0, 1),
            ("Sanders", 4, 12),
            ("Steyer", 0, 1),
            ("Warren", 26, 60),
            ("Williamson", 0, 1),
            ("Yang", 0, 2),
        ],
    );

    market.player_ranges(
        "ijime-deactivated20150440",
        vec![
            ("Biden", 10, 15),
            ("Sanders", 10, 20),
            ("Warren", 10, 34),
            ("Klobuchar", 0, 1),
            ("Buttigieg", 10, 30),
            ("Gabbard", 3, 6),
            ("Booker", 2, 5),
            ("Williamson", 2, 5),
            ("Yang", 3, 15),
        ],
    );
}

fn setup_session7(market: &mut Market) {
    market.new_player("thomas-midgley-did-nothing-wrong");
    market.new_player("goatsgomoo");

    market.player_ranges(
        "tremorbond",
        vec![
            ("Biden", 33, 39),
            ("Warren", 24, 28),
            ("Sanders", 18, 22),
            ("Buttigieg", 10, 12),
            ("Yang", 1, 3),
            ("Bloomberg", 0, 1),
            ("Klobuchar", 1, 2),
        ],
    );

    market.player_ranges(
        "thomas-midgley-did-nothing-wrong",
        vec![
            ("Biden", 25, 40),
            ("Sanders", 15, 35),
            ("Warren", 35, 55),
            ("Yang", 1, 5),
        ],
    );

    market.player_ranges(
        "von-hresvelg",
        vec![
            ("Warren", 30, 80),
            ("Sanders", 25, 60),
            ("Yang", 10, 40),
            ("Buttigieg", 2, 10),
            ("Biden", 15, 25),
        ],
    );

    market.player_ranges(
        "utilitymonstermash",
        vec![
            ("Bennet", 0, 1),
            ("Biden", 27, 60),
            ("Bloomberg", 6, 22),
            ("Booker", 0, 1),
            ("Buttigieg", 9, 29),
            ("Castro", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 2),
            ("Patrick", 0, 1),
            ("Sanders", 4, 12),
            ("Steyer", 0, 1),
            ("Warren", 27, 60),
            ("Williamson", 0, 1),
            ("Yang", 0, 2),
        ],
    );

    market.player_ranges(
        "ijime-deactivated20150440",
        vec![("Yang", 3, 100), ("Gabbard", 2, 100), ("Booker", 2, 100)],
    );

    market.player_ranges(
        "goatsgomoo",
        vec![
            ("Biden", 40, 75),
            ("Sanders", 15, 75),
            ("Warren", 15, 40),
            ("Buttigieg", 0, 1),
            ("Yang", 0, 1),
            ("Gabbard", 0, 1),
            ("Castro", 0, 1),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Klobuchar", 0, 1),
        ],
    );
}

fn setup_session8(market: &mut Market) {
    market.player_ranges(
        "firebendinglemur",
        vec![
            ("Biden", 70, 80),
            ("Warren", 8, 10),
            ("Sanders", 5, 6),
            ("Gabbard", 1, 2),
            ("Buttigieg", 0, 1),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "thomas-midgley-did-nothing-wrong",
        vec![
            ("Biden", 25, 30),
            ("Sanders", 25, 45),
            ("Warren", 30, 45),
            ("Yang", 1, 5),
        ],
    );

    market.player_ranges(
        "utilitymonstermash",
        vec![
            ("Bennet", 0, 1),
            ("Biden", 32, 65),
            ("Bloomberg", 3, 22),
            ("Booker", 0, 1),
            ("Buttigieg", 8, 29),
            ("Castro", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 2),
            ("Patrick", 0, 1),
            ("Sanders", 4, 12),
            ("Steyer", 0, 1),
            ("Warren", 27, 60),
            ("Williamson", 0, 1),
            ("Yang", 0, 3),
        ],
    );
}

fn setup_session9(market: &mut Market) {
    market.new_player("holomanga");
    market.rename_player("von-hresvelg", "princesse266");
    market.contract_outcome("Castro", false);

    market.player_ranges(
        "holomanga",
        vec![
            ("Biden", 24, 48),
            ("Warren", 12, 24),
            ("Sanders", 12, 24),
            ("Buttigieg", 6, 12),
            ("Bloomberg", 6, 12),
            ("Yang", 3, 6),
            ("Klobuchar", 1, 3),
            ("Bennet", 0, 1),
            ("Booker", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Patrick", 0, 1),
            ("Steyer", 0, 1),
            ("Williamson", 0, 1),
        ],
    );

    market.player_ranges(
        "the-moti",
        vec![
            ("Biden", 30, 37),
            ("Sanders", 19, 25),
            ("Warren", 12, 17),
            ("Buttigieg", 10, 14),
            ("Bloomberg", 3, 13),
            ("Klobuchar", 3, 6),
            ("Yang", 2, 10),
            ("Gabbard", 0, 1),
            ("Booker", 0, 1),
        ],
    );
}

fn setup_session10(market: &mut Market) {
    market.new_player("qualiteagirl");
    market.contract_outcome("Williamson", false);

    market.player_ranges(
        "ouroborostriumphant",
        vec![
            ("Biden", 48, 52),
            ("Sanders", 18, 22),
            ("Warren", 13, 17),
            ("Buttigieg", 13, 17),
            ("Yang", 0, 5),
            ("Bloomberg", 0, 3),
            ("Booker", 0, 2),
            ("Gabbard", 0, 1),
            ("Patrick", 0, 1),
            ("Steyer", 0, 1),
        ],
    );

    market.player_ranges(
        "placid-platypus",
        vec![
            ("Biden", 37, 54),
            ("Warren", 20, 30),
            ("Sanders", 12, 16),
            ("Buttigieg", 10, 16),
            ("Yang", 1, 6),
            ("Klobuchar", 1, 4),
            ("Booker", 0, 1),
            ("Patrick", 0, 2),
            ("Bloomberg", 0, 1),
        ],
    );

    market.player_ranges(
        "squareallworthy",
        vec![
            ("Biden", 49, 50),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Buttigieg", 5, 11),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 1),
            ("Patrick", 0, 1),
            ("Sanders", 12, 15),
            ("Warren", 25, 30),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "irradiate-space",
        vec![
            ("Biden", 20, 31),
            ("Bloomberg", 0, 11),
            ("Buttigieg", 10, 20),
            ("Sanders", 30, 41),
            ("Warren", 30, 57),
            ("Yang", 0, 2),
        ],
    );

    market.player_ranges(
        "ijime-deactivated20150440",
        vec![("Yang", 3, 100), ("Gabbard", 2, 100)],
    );

    market.player_ranges(
        "goatsgomoo",
        vec![
            ("Biden", 70, 95),
            ("Sanders", 15, 70),
            ("Warren", 15, 70),
            ("Buttigieg", 0, 1),
            ("Yang", 0, 1),
            ("Gabbard", 0, 1),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Klobuchar", 0, 1),
        ],
    );

    market.player_ranges(
        "holomanga",
        vec![
            ("Biden", 32, 64),
            ("Sanders", 16, 32),
            ("Warren", 8, 16),
            ("Bloomberg", 8, 16),
            ("Buttigieg", 4, 8),
            ("Yang", 2, 4),
            ("Klobuchar", 1, 2),
            ("Bennet", 0, 1),
            ("Booker", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Patrick", 0, 1),
            ("Steyer", 0, 1),
        ],
    );

    market.player_ranges(
        "utilitymonstermash",
        vec![
            ("Bennet", 0, 1),
            ("Biden", 39, 69),
            ("Bloomberg", 1, 22),
            ("Booker", 0, 2),
            ("Buttigieg", 9, 55),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 2),
            ("Klobuchar", 0, 2),
            ("Patrick", 0, 1),
            ("Sanders", 5, 35),
            ("Steyer", 0, 1),
            ("Warren", 30, 60),
            ("Yang", 0, 3),
        ],
    );

    market.player_ranges(
        "qualiteagirl",
        vec![
            ("Biden", 50, 75),
            ("Sanders", 25, 37),
            ("Warren", 12, 25),
            ("Buttigieg", 5, 12),
            ("Bennet", 0, 1),
            ("Bloomberg", 0, 1),
            ("Booker", 0, 1),
            ("Delaney", 0, 1),
            ("Gabbard", 0, 1),
            ("Klobuchar", 0, 1),
            ("Patrick", 0, 1),
            ("Steyer", 0, 1),
            ("Yang", 0, 1),
        ],
    );

    market.player_ranges(
        "cromulentenough",
        vec![
            ("Sanders", 20, 30),
            ("Warren", 10, 20),
            ("Biden", 35, 45),
            ("Buttigieg", 10, 20),
            ("Gabbard", 1, 4),
            ("Bloomberg", 1, 4),
            ("Yang", 1, 4),
            ("Klobuchar", 0, 2),
            ("Bennet", 0, 2),
            ("Booker", 0, 2),
            ("Patrick", 0, 2),
            ("Steyer", 0, 2),
            ("Delaney", 0, 2),
        ],
    );

    market.player_ranges(
        "the-moti",
        vec![
            ("Biden", 33, 42),
            ("Sanders", 24, 30),
            ("Warren", 10, 13),
            ("Bloomberg", 5, 13),
            ("Buttigieg", 8, 10),
            ("Yang", 2, 5),
            ("Klobuchar", 1, 5),
            ("Gabbard", 0, 1),
            ("Booker", 0, 1),
        ],
    );
}
