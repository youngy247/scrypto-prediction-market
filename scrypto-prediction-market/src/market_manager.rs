use scrypto::prelude::*;
use crate::prediction_market::prediction_market::PredictionMarket;


#[blueprint]
mod market_manager {
    struct MarketManager {
        markets: HashMap<String, Global<PredictionMarket>>,
    }    

    impl MarketManager {
        pub fn new() -> Global<MarketManager> {
            Self {
                markets: HashMap::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn instantiate_prediction_market(&mut self, market_id: String, outcomes_str: String, odds_str: String) {
            let market = PredictionMarket::instantiate_prediction_market(outcomes_str, odds_str);
            self.markets.insert(market_id, market);
        }        
        
        pub fn get_market(&self, market_id: String) -> Option<Global<PredictionMarket>> {
            self.markets.get(&market_id).cloned()
        }        

        pub fn list_all_markets(&self) -> Vec<String> {
            self.markets.keys().cloned().collect()
        }
        
    }
}
