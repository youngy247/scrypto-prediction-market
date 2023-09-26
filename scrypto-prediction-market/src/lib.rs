use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    struct PredictionMarket {
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        total_staked: Decimal,
    }
}
