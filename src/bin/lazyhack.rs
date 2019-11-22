use std::cmp::{max, min};
use std::collections::BTreeMap;

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
    player_id: PlayerID,
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
    exposures: BTreeMap<PlayerID, Exposure>,
    ious: Vec<Iou>,
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
    }

    pub fn player_ranges(&mut self, name: &str, ranges: Vec<(&str, Price, Price)>) {
        let player_id = *self.player_names.get(name).unwrap();
        self.get_player_mut(player_id).clear_ranges();
        for (contract_name, low, high) in ranges {
            let contract_id = match self.contract_names.get(contract_name) {
                Some(contract_id) => *contract_id,
                None => panic!("contract does not exist: {}", contract_name),
            };
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
        let mut exposure = Exposure::new(player_id);
        for iou in &self.ious {
            exposure.apply_iou(iou);
        }
        exposure
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
        let mut session = Session::new();
        for (_player_name, player_id) in &self.player_names {
            let exposure = self.calc_exposure(*player_id);
            session.exposures.insert(*player_id, exposure);
        }

        loop {
            let offers = session.find_offers(self);
            let trades = session.find_trades(self, &offers);

            if let Some((_spread, _contract_name, contract_id, buyer_id, high, seller_id, low)) =
                trades.into_iter().next()
            {
                let price = (low + high) / 2;

                //println!("trade: {}", contract_id);
                //println!("price: {}", price);
                //println!();

                let buyer_max_amount = self.player_max_buy_amount(&session, buyer_id, contract_id);
                let buyer_max_units = buyer_max_amount / price;

                //println!("buyer: {}", buyer_id);
                //println!("buyer_max_amount = {}", buyer_max_amount);
                //println!("buyer_max_units = {}", buyer_max_units);
                //println!();

                let seller_max_amount =
                    self.player_max_sell_amount(&session, seller_id, contract_id);
                let seller_max_units = seller_max_amount / (100 - price);

                //println!("seller: {}", seller_id);
                //println!("seller_max_amount = {}", seller_max_amount);
                //println!("seller_max_units = {}", seller_max_units);
                //println!();

                //println!("buyer_maxed_out = {}", buyer_maxed_out);
                //println!("seller_maxed_out = {}", seller_maxed_out);
                //println!();

                let trade_units = min(buyer_max_units, seller_max_units);

                if !(trade_units > 0) {
                    panic!("trade_units = {}", trade_units);
                }

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

                //println!(
                //    "{} {} units @ {} : {} -> {}",
                //    contract_id, trade_units, price, seller_id, buyer_id
                //);
                //println!();

                session.apply_iou(seller_iou);
                session.apply_iou(buyer_iou);

                self.check_credit_failure(buyer_id);
                self.check_credit_failure(seller_id);
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
    }
}

impl Session {
    pub fn new() -> Self {
        let exposures = BTreeMap::new();
        let ious = Vec::new();
        Session { exposures, ious }
    }

    pub fn apply_iou(&mut self, iou: Iou) {
        self.exposures
            .get_mut(&iou.issuer_id)
            .unwrap()
            .apply_iou(&iou);
        self.exposures
            .get_mut(&iou.holder_id)
            .unwrap()
            .apply_iou(&iou);
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
            if let Some((high, buyers)) = contract_offers.buy.iter().rev().next() {
                if let Some((low, sellers)) = contract_offers.sell.iter().next() {
                    if low <= high {
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

impl ContractOffers {
    pub fn new() -> Self {
        let buy = BTreeMap::new();
        let sell = BTreeMap::new();
        ContractOffers { buy, sell }
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

    pub fn clear_ranges(&mut self) {
        self.ranges.clear();
    }

    pub fn set_range(&mut self, contract_id: ContractID, low: Price, high: Price) {
        self.ranges.insert(contract_id, (low, high));
    }
}

impl Exposure {
    pub fn new(player_id: PlayerID) -> Self {
        let exposure = BTreeMap::new();
        let neg_exposure = BTreeMap::new();
        Exposure {
            player_id,
            exposure,
            neg_exposure,
        }
    }

    pub fn apply_iou(&mut self, iou: &Iou) {
        if iou.issuer_id == self.player_id {
            if iou.condition {
                self.add_exposure(iou.contract_id, iou.amount);
            } else {
                self.add_neg_exposure(iou.contract_id, iou.amount);
            }
        } else if iou.holder_id == self.player_id {
            if iou.condition {
                self.add_exposure(iou.contract_id, -iou.amount);
            } else {
                self.add_neg_exposure(iou.contract_id, -iou.amount);
            }
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

    // Sun 10 Nov 2019
    setup_session1(&mut market);

    market.dump();
    market.session();
    market.dump_aftermath();

    // Sat 16 Nov 2019
    setup_session2(&mut market);

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

    market.increment_credit(1000);

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
