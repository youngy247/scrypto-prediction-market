use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    struct PredictionMarket {
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        total_staked: Decimal,
    }

    impl PredictionMarket {
        pub fn instantiate_prediction_market(outcomes: Vec<String>) -> Global<PredictionMarket> {
            let mut outcome_tokens = Vec::new();

            for outcome in &outcomes {
                let token_name = format!("PredictionToken_{}", outcome);
                let token = ResourceBuilder::new_fungible(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => token_name.clone(), locked;
                            "symbol" => outcome.clone(), locked;
                            "description" => format!("Token for predicting {}", outcome), locked;
                        }
                    ))
                    .mint_initial_supply(0); // Initialize with zero tokens
                outcome_tokens.push(Vault::with_bucket(token.into()));
            }

            Self {
                outcome_tokens,
                outcomes,
                total_staked: Decimal::from(0),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn list_outcomes(&self) -> Vec<String> {
            self.outcomes.clone()
        }
    }
}