use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    struct PredictionMarket {
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        total_staked: Decimal,
        xrd_vault: Vault,
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
                xrd_vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn list_outcomes(&self) -> Vec<String> {
            self.outcomes.clone()
        }

        pub fn resolve_market(&mut self, winning_outcome: u32) -> Vec<(String, Decimal)> {
            if (winning_outcome as usize) < self.outcome_tokens.len() {
                // Calculate the reward for each participant
                let mut rewards = Vec::new();
                for (index, outcome_token) in self.outcome_tokens.iter_mut().enumerate() {
                    let bet_amount: Bucket = outcome_token.take_all();

                    let outcome = &self.outcomes[index];
                    let reward = if index == winning_outcome as usize {
                        // If it's the winning outcome, distribute the entire pot to participants
                        self.total_staked.clone()
                    } else {
                        // Otherwise, return the bet amount as is
                        Decimal::from(bet_amount.amount())
                    };

                    rewards.push((outcome.clone(), reward));
                }

                // Reset the market after resolution
                for t in &mut self.outcome_tokens {
                    drop(t.take_all()); // Drop the Bucket value explicitly
                }
                self.total_staked = Decimal::from(0);

                return rewards;
            }

            // Invalid winning outcome
            Vec::new()
        }

        pub fn place_bet(&mut self, outcome: String, bet_amount: Decimal, mut user_xrd_vault: Vault) -> bool {
            // Find the index of the outcome, if it exists in the outcomes vector.
            if let Some(index) = self.outcomes.iter().position(|o| o == &outcome) {
                let outcome_token = &mut self.outcome_tokens[index];
                
                // Directly take the XRD tokens from the user's vault.
                let taken_bucket = user_xrd_vault.take(bet_amount);
                
                // Put the taken XRD tokens into the corresponding outcome's vault.
                outcome_token.put(taken_bucket);
                
                self.total_staked += bet_amount;
                return true;
            } else {
                // If the outcome doesn't exist, return false.
                false
            }
        }
        
    }
            
}
