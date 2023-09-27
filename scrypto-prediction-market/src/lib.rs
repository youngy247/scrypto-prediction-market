use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    struct PredictionMarket {
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        total_staked: Decimal,
        xrd_vault: Vault,
            /// A Vec of tuples. Each tuple consists of an Account hash and a balance.
        users: Vec<(String, Decimal)>,
    }

    impl PredictionMarket {
        pub fn instantiate_prediction_market(outcomes_str: String) -> Global<PredictionMarket> {
            let outcomes: Vec<String> = outcomes_str.split(',').map(|s| s.trim().to_string()).collect();
            
            let mut outcome_tokens = Vec::new();
            for _ in &outcomes {
                outcome_tokens.push(Vault::new(XRD)); // Create a new XRD vault for each outcome
            }
        
            Self {
                outcome_tokens,
                outcomes,
                total_staked: Decimal::from(0),
                xrd_vault: Vault::new(XRD),
                users: Vec::new(), // Initialize the vector
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }
        

        pub fn list_outcomes(&self) -> Vec<String> {
            self.outcomes.clone()
        }
        pub fn get_total_staked(&self) -> Decimal {
            self.total_staked.clone()
        }

        pub fn get_outcome_balance(&self, outcome: String) -> Option<Decimal> {
            self.outcomes.iter().position(|o| o == &outcome).map(|index| {
                Decimal::from(self.outcome_tokens[index].amount())
            })
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

            /// Adds or updates a user's balance. If the user already exists, their balance will be increased by `amount`.
            /// Otherwise, a new user entry is created with the specified `amount`.
        pub fn add_or_update_user_balance(&mut self, user_hash: String, amount: Decimal) {
            if let Some((_, balance)) = self.users.iter_mut().find(|(u, _)| u == &user_hash) {
                *balance += amount;
            } else {
                self.users.push((user_hash, amount));
            }
        }
    }
}
