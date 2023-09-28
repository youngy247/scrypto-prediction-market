use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    pub struct PredictionMarket {
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        odds: Vec<Decimal>,   
        total_staked: Decimal,
        bets: Vec<(String, String, Decimal)>,
        xrd_vault: Vault,
        user_vaults: HashMap<String, Vault>,
        market_resolved: bool,
    }


    impl PredictionMarket {
        pub fn instantiate_prediction_market(outcomes_str: String, odds_str: String) -> Global<PredictionMarket> {
            let outcomes: Vec<String> = outcomes_str.split(',').map(|s| s.trim().to_string()).collect();
            let odds: Vec<Decimal> = odds_str.split(',')
                .map(|s| Decimal::from_str(s.trim()).expect("Failed to parse odds as Decimal"))
                .collect();
        
            assert_eq!(outcomes.len(), odds.len(), "Number of odds should match the number of outcomes.");
        
            let mut outcome_tokens = Vec::new();
            for _ in &outcomes {
                outcome_tokens.push(Vault::new(XRD)); // Create a new XRD vault for each outcome
            }
            
            Self {
                outcome_tokens,
                outcomes,
                odds,  
                total_staked: Decimal::from(0),
                bets: Vec::new(),
                xrd_vault: Vault::new(XRD),
                user_vaults: HashMap::new(),
                market_resolved: false
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

        pub fn get_outcome_balance(&self, outcome: String) -> Decimal {
            assert!(self.outcomes.contains(&outcome), "Outcome does not exist.");
            
            let index = self.outcomes.iter().position(|o| o == &outcome).expect("Outcome not found.");
            Decimal::from(self.outcome_tokens[index].amount())
        }
        

        pub fn place_bet(&mut self, user_hash: String, outcome: String, payment: Bucket) -> Result<(), String> {
            if self.market_resolved {
                return Err("Market has already been resolved.".to_string());
            }
            
            let bet_amount = payment.amount();
            if bet_amount <= Decimal::from(0) {
                return Err("Invalid bet amount.".to_string());
            }

            // Check if user has a vault, if not create one.
            if !self.user_vaults.contains_key(&user_hash) {
                self.user_vaults.insert(user_hash.clone(), Vault::new(XRD));
            }

            match self.outcomes.iter().position(|o| o == &outcome) {
                Some(index) => {
                    let outcome_token = &mut self.outcome_tokens[index];
                    outcome_token.put(payment);
                    self.total_staked += bet_amount;

                    self.bets.push((user_hash, outcome, bet_amount));  // Record the bet
                    Ok(())
                },
                None => Err("Outcome not found.".to_string())
            }
        }

        pub fn deposit_to_xrd_vault(&mut self, deposit: Bucket) {

            self.xrd_vault.put(deposit);
        }

        pub fn get_xrd_vault_balance(&self) -> Decimal {
            Decimal::from(self.xrd_vault.amount())
        }

        pub fn resolve_market(&mut self, winning_outcome: u32) -> Result<Vec<(String, Decimal)>, String> {
            assert!((winning_outcome as usize) < self.outcome_tokens.len(), "Winning outcome is out of bounds.");
            assert!(!self.market_resolved, "Market has already been resolved.");

            println!("Resolving market for winning outcome: {}", winning_outcome);
        
            let mut rewards = Vec::new();
        
            for (index, outcome_vault) in self.outcome_tokens.iter_mut().enumerate() {
                if index == winning_outcome as usize {
                    continue; // Skip the winning vault for now
                }
        
                let tokens = outcome_vault.take_all();
                println!("Tokens taken from losing vault {}: {:?}", index, tokens);
        
                // Transfer tokens from losing vaults to the xrd_vault.
                self.xrd_vault.put(tokens);
            }
        
            // Print the total amount now in the xrd_vault
            println!("Total amount in xrd_vault after transferring from losing vaults: {}", self.xrd_vault.amount());
        
            // The total amount staked for the winning outcome.
            let total_winning_staked = self.outcome_tokens[winning_outcome as usize].amount();
            println!("Total amount staked for the winning outcome {}: {}", winning_outcome, total_winning_staked);
        
            for (user, bet_outcome, bet_amt) in &self.bets {
                if bet_outcome == &self.outcomes[winning_outcome as usize] {
                    // Calculate the user's proportion of the total staked amount for the winning outcome
                    let user_proportion = *bet_amt / total_winning_staked;
        
                    // Print user's proportion
                    println!("User {} proportion of total winning stake: {}", user, user_proportion);
        
                    // Calculate the reward based on the odds and the user's proportion of the winning stake
                    let user_reward = *bet_amt * self.odds[winning_outcome as usize];
                    
                    // Print calculated reward for the user
                    println!("Calculated reward for user {}: {}", user, user_reward);
        
                    rewards.push((user.clone(), user_reward));
                    
                    // Extract reward from xrd_vault
                    let reward_bucket = self.xrd_vault.take(user_reward);
        
                    // TODO: Transfer reward_bucket to the user's vault or account. You'll need a mechanism to do this based on user_hash.
                    // Ideally, you'll use something like `user_vault.put(reward_bucket)`;
        
                    // Transfer reward_bucket to the user's vault using the HashMap.
                    if let Some(user_vault) = self.user_vaults.get_mut(user) {
                        user_vault.put(reward_bucket);
                    }
                }
            }
        
            self.total_staked = Decimal::from(0);
            println!("Reset total staked to 0.");
        
            self.market_resolved = true; 
            Ok(rewards)
        }

        // Add a new method for users to claim their rewards from their vaults.
        pub fn claim_reward(&mut self, user_hash: String) -> Option<Bucket> {
            self.user_vaults.get_mut(&user_hash).map(|vault| vault.take_all())
        }
    
    }        
}
