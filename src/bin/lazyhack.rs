use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

type Price = i32;

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
}

struct Player {
    name: String,
    ranges: BTreeMap<ContractID, (Price, Price)>,
    credit_limit: Price,
}

struct Exposure {
    exposure: BTreeMap<ContractID, Price>,
    neg_exposure: BTreeMap<ContractID, Price>,
}

struct Iou {
    pub issuer_id: PlayerID,
    pub holder_id: PlayerID,
    pub contract_id: ContractID,
    pub condition: bool,
    pub amount: Price,
}

struct Session {
    offers: BTreeMap<ContractID, Offers>,
}

struct Offers {
    buy: BTreeMap<Price, BTreeSet<PlayerID>>,
    sell: BTreeMap<Price, BTreeSet<PlayerID>>,
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

    pub fn get_player(&self, player_id: PlayerID) -> &Player {
        &self.players[player_id.0]
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

    pub fn contract(&mut self, name: &str) {
        let contract = Contract::new(name);
        self.add_contract(contract);
    }

    pub fn player_ranges(&mut self, name: &str, ranges: Vec<(&str, Price, Price)>) {
        let mut player = Player::new(name);
        for (contract_name, low, high) in ranges {
            if let Some(contract_id) = self.contract_names.get(contract_name) {
                if !(0 <= low && low < high && high <= 100) {
                    panic!("invalid range: {}..{}", low, high);
                }
                player.set_range(*contract_id, low, high);
            } else {
                panic!("contract does not exist: {}", contract_name);
            }
        }
        self.add_player(player);
    }

    pub fn dump(&self) {
        println!("CONTRACTS ({})", self.contracts.len());
        for (contract_name, _contract_id) in &self.contract_names {
            println!(" - {}", contract_name);
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
            let contract_name = &self.get_contract(iou.contract_id).name;
            if iou.condition {
                println!(
                    " - {} owes {} to {} if {}",
                    issuer_name, iou.amount, holder_name, contract_name
                );
            } else {
                println!(
                    " - {} owes {} to {} if NOT {}",
                    issuer_name, iou.amount, holder_name, contract_name
                );
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
                    println!(" - {}: {}", contract.name, outcome);
                }
                if otherwise != 0 {
                    println!(" - other: {}", otherwise);
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
                if iou.condition {
                    exposure.add_exposure(iou.contract_id, iou.amount);
                } else {
                    exposure.add_neg_exposure(iou.contract_id, iou.amount);
                }
            } else if iou.holder_id == player_id {
                if iou.condition {
                    exposure.add_exposure(iou.contract_id, -iou.amount);
                } else {
                    exposure.add_neg_exposure(iou.contract_id, -iou.amount);
                }
            }
        }
        exposure
    }

    pub fn calc_exposure_to_contract(
        &self,
        player_id: PlayerID,
        contract_id: ContractID,
        condition: bool,
    ) -> Price {
        let exposure = self.calc_exposure(player_id);
        if condition {
            exposure.total_exposure_to_contract(contract_id)
        } else {
            exposure.total_exposure_to_contract_neg(contract_id)
        }
    }

    pub fn check_credit_failure(&self, player_id: PlayerID) {
        let player = self.get_player(player_id);
        let exposure = self.calc_exposure(player_id);
        for contract_id in self.contract_names.values() {
            let ex = exposure.total_exposure_to_contract(*contract_id);
            if ex > player.credit_limit {
                let contract = self.get_contract(*contract_id);
                panic!(
                    "{}: {} exposed {} > {}",
                    player.name, contract.name, ex, player.credit_limit
                );
            }
        }
    }

    pub fn session(&mut self) {
        let mut session = Session::new();
        for (_player_name, player_id) in &self.player_names {
            let player = self.get_player(*player_id);
            for (contract_id, (low, high)) in &player.ranges {
                session.add_offers(*contract_id, *player_id, *low, *high);
            }
        }

        loop {
            let trades = session.find_trades(self);

            if let Some((_spread, _contract_name, contract_id, buyer_id, high, seller_id, low)) =
                trades.into_iter().next()
            {
                let price = (low + high) / 2;

                //println!("trade: {}", contract_id);
                //println!("price: {}", price);
                //println!();

                let buyer_credit_limit = self.get_player(buyer_id).credit_limit;
                let buyer_exposure = self.calc_exposure_to_contract(buyer_id, contract_id, false);
                let buyer_max_amount = max(0, buyer_credit_limit - buyer_exposure);
                let buyer_max_units = buyer_max_amount / price;

                //println!("buyer: {}", buyer_id);
                //println!("buyer_credit_limit = {}", buyer_credit_limit);
                //println!("buyer_exposure = {}", buyer_exposure);
                //println!("buyer_max_amount = {}", buyer_max_amount);
                //println!("buyer_max_units = {}", buyer_max_units);
                //println!();

                let seller_credit_limit = self.get_player(seller_id).credit_limit;
                let seller_exposure = self.calc_exposure_to_contract(seller_id, contract_id, true);
                let seller_max_amount = max(0, seller_credit_limit - seller_exposure);
                let seller_max_units = seller_max_amount / (100 - price);

                //println!("seller: {}", seller_id);
                //println!("seller_credit_limit = {}", seller_credit_limit);
                //println!("seller_exposure = {}", seller_exposure);
                //println!("seller_max_amount = {}", seller_max_amount);
                //println!("seller_max_units = {}", seller_max_units);
                //println!();

                let buyer_maxed_out = buyer_max_units <= seller_max_units;
                let seller_maxed_out = seller_max_units <= buyer_max_units;

                //println!("buyer_maxed_out = {}", buyer_maxed_out);
                //println!("seller_maxed_out = {}", seller_maxed_out);
                //println!();

                let trade_units = min(buyer_max_units, seller_max_units);

                if trade_units > 0 {
                    let seller_iou = Iou {
                        issuer_id: seller_id,
                        holder_id: buyer_id,
                        contract_id: contract_id,
                        condition: true,
                        amount: trade_units * (100 - price),
                    };

                    let buyer_iou = Iou {
                        issuer_id: buyer_id,
                        holder_id: seller_id,
                        contract_id: contract_id,
                        condition: false,
                        amount: trade_units * price,
                    };

                    //println!("{:?}", seller_iou);
                    //println!();
                    //println!("{:?}", buyer_iou);
                    //println!();

                    self.ious.push(seller_iou);
                    self.ious.push(buyer_iou);

                    self.check_credit_failure(buyer_id);
                    self.check_credit_failure(seller_id);

                    //println!(
                    //    "{} {} units @ {} : {} -> {}",
                    //    contract_id, trade_units, price, seller_id, buyer_id
                    //);
                    //println!();
                }

                if buyer_maxed_out {
                    session
                        .offers
                        .get_mut(&contract_id)
                        .unwrap()
                        .remove_buy_offer(high, buyer_id);
                }

                if seller_maxed_out {
                    session
                        .offers
                        .get_mut(&contract_id)
                        .unwrap()
                        .remove_sell_offer(low, seller_id);
                }
            } else {
                //println!("no trades!");
                //println!();
                break;
            }
        }

        let spreads = session.find_spreads(self);
        println!("SPREADS ({})", spreads.len());
        for (contract_name, (low, high)) in spreads.into_iter() {
            println!(" - {} {}-{}", contract_name, low, high);
        }
        println!();
    }
}

impl Session {
    pub fn new() -> Self {
        let offers = BTreeMap::new();
        Session { offers }
    }

    pub fn add_offers(
        &mut self,
        contract_id: ContractID,
        player_id: PlayerID,
        low: Price,
        high: Price,
    ) {
        let offers = self
            .offers
            .entry(contract_id)
            .or_insert_with(|| Offers::new());
        offers
            .buy
            .entry(low)
            .and_modify(|players| {
                players.insert(player_id);
            })
            .or_insert_with(|| {
                let mut players = BTreeSet::new();
                players.insert(player_id);
                players
            });
        offers
            .sell
            .entry(high)
            .and_modify(|players| {
                players.insert(player_id);
            })
            .or_insert_with(|| {
                let mut players = BTreeSet::new();
                players.insert(player_id);
                players
            });
    }

    pub fn find_trades(
        &self,
        market: &Market,
    ) -> Vec<(Price, String, ContractID, PlayerID, Price, PlayerID, Price)> {
        let mut trades = Vec::new();
        for (contract_id, offers) in &self.offers {
            let contract = market.get_contract(*contract_id);
            if let Some((high, buyers)) = offers.buy.iter().rev().next() {
                if let Some((low, sellers)) = offers.sell.iter().next() {
                    if low <= high {
                        let mut buyers: Vec<PlayerID> = buyers.iter().map(|x| *x).collect();
                        let mut sellers: Vec<PlayerID> = sellers.iter().map(|x| *x).collect();
                        // FIXME
                        buyers.sort_by_key(|player_id| &market.get_player(*player_id).name);
                        sellers.sort_by_key(|player_id| &market.get_player(*player_id).name);
                        //let salt = player_id.chars().fold(0, |x, c| x ^ (c as i32)); // FIXME
                        let buyer = buyers.iter().next().unwrap(); // FIXME
                        let seller = sellers.iter().next().unwrap(); // FIXME
                        let spread = high - low;
                        trades.push((
                            spread,
                            contract.name.clone(), // for consistent ordering
                            *contract_id,
                            buyer.clone(),
                            *high,
                            seller.clone(),
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

    pub fn find_spreads(&self, market: &Market) -> BTreeMap<String, (Price, Price)> {
        let mut spreads = BTreeMap::new();
        for (contract_id, offers) in &self.offers {
            let contract = market.get_contract(*contract_id);
            let buy = *offers.buy.keys().rev().next().unwrap_or(&0);
            let sell = *offers.sell.keys().next().unwrap_or(&100);
            if sell <= buy {
                panic!("you said no trades! {} {} {}", contract.name, buy, sell);
            }
            spreads.insert(contract.name.clone(), (buy, sell));
        }
        spreads
    }
}

impl Offers {
    pub fn new() -> Self {
        let buy = BTreeMap::new();
        let sell = BTreeMap::new();
        Offers { buy, sell }
    }

    pub fn remove_buy_offer(&mut self, price: Price, player_id: PlayerID) {
        let buyers = self.buy.get_mut(&price).unwrap();
        buyers.remove(&player_id);
        if buyers.is_empty() {
            self.buy.remove(&price);
        }
    }

    pub fn remove_sell_offer(&mut self, price: Price, player_id: PlayerID) {
        let sellers = self.sell.get_mut(&price).unwrap();
        sellers.remove(&player_id);
        if sellers.is_empty() {
            self.sell.remove(&price);
        }
    }
}

impl Contract {
    pub fn new(name: impl Into<String>) -> Self {
        Contract { name: name.into() }
    }
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        let ranges = BTreeMap::new();
        let credit_limit = 1000;
        Player {
            name: name.into(),
            ranges,
            credit_limit,
        }
    }

    pub fn set_range(&mut self, contract_id: ContractID, low: Price, high: Price) {
        self.ranges.insert(contract_id, (low, high));
    }
}

impl Exposure {
    pub fn new() -> Self {
        let exposure = BTreeMap::new();
        let neg_exposure = BTreeMap::new();
        Exposure {
            exposure,
            neg_exposure,
        }
    }

    fn exposure(&self, contract_id: ContractID) -> Price {
        *self.exposure.get(&contract_id).unwrap_or(&0)
    }

    fn neg_exposure(&self, contract_id: ContractID) -> Price {
        *self.neg_exposure.get(&contract_id).unwrap_or(&0)
    }

    fn total_neg_exposure(&self) -> Price {
        self.neg_exposure.values().sum()
    }

    // exposure to P:
    //  - how much debt we owe (net assets) conditional on P
    //  - plus how much debt we owe conditional on ~Q, where Q \= P
    pub fn total_exposure_to_contract(&self, contract_id: ContractID) -> Price {
        self.exposure(contract_id) + self.total_neg_exposure() - self.neg_exposure(contract_id)
    }

    // exposure to ~P:
    // - how much debt we owe (do *not* count assets!) conditional on ~Q for all Q
    // - plus biggest debt we owe conditional on Q where Q \= P
    pub fn total_exposure_to_contract_neg(&self, contract_id: ContractID) -> Price {
        self.total_exposure_to_neg()
            + *self
                .exposure
                .iter()
                .filter(|(contract_id0, amount)| {
                    **contract_id0 != contract_id && amount.is_positive()
                })
                .map(|(_contract_id, amount)| amount)
                .max()
                .unwrap_or(&0)
    }

    // worst case exposure to ~Q for all Q:
    // - how much debt we owe (do *not* count assets!) conditional on ~Q for all Q
    pub fn total_exposure_to_neg(&self) -> Price {
        self.neg_exposure.values().filter(|x| x.is_positive()).sum()
    }

    pub fn outcome(&self, contract_id: ContractID) -> Price {
        -self.total_exposure_to_contract(contract_id)
    }

    pub fn otherwise_outcome(&self) -> Price {
        -self.total_neg_exposure()
    }

    pub fn add_exposure(&mut self, contract_id: ContractID, amount: Price) {
        self.exposure
            .entry(contract_id)
            .and_modify(|total| *total += amount)
            .or_insert(amount);
    }

    pub fn add_neg_exposure(&mut self, contract_id: ContractID, amount: Price) {
        self.neg_exposure
            .entry(contract_id)
            .and_modify(|total| *total += amount)
            .or_insert(amount);
    }
}

fn main() {
    let mut market = Market::new();

    market.contract("Sanders");
    market.contract("Warren");
    market.contract("Biden");
    market.contract("Buttigieg");
    market.contract("Harris");
    market.contract("Steyer");
    market.contract("Yang");
    market.contract("Gillibrand");
    market.contract("Gabbard");
    market.contract("Williamson");
    market.contract("Booker");
    market.contract("Klobuchar");
    market.contract("Castro");

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

    market.dump();
    market.session();
    market.dump_aftermath();
}
