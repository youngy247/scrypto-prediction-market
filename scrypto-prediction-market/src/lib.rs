use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    struct PredictionMarket {
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        odds: Vec<Decimal>,   
        total_staked: Decimal,
        bets: Vec<(String, String, Decimal)>,
        xrd_vault: Vault,
        user_vaults: HashMap<String, Vault>,
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
        

        pub fn deposit_to_xrd_vault(&mut self, deposit: Bucket) {

            self.xrd_vault.put(deposit);
        }

        pub fn deposit(&mut self, account_id: String, payment: Bucket) -> Result<(), String> {
            // Try to take the deposit amount from the user's default payment bucket
            let deposit_amount = payment.amount();
            
            // Check if the deposit was successful (i.e., the user had enough balance)
            if deposit_amount > Decimal::from(0) {
            // Add the payment bucket to the component's xrd_vault
            self.xrd_vault.put(payment);

            // Add or update the user's balance in the users Vec.
            self.add_or_update_user_balance(account_id.clone(), deposit_amount);

            Ok(())
        } else {
            Err(format!("Invalid or insufficient XRD payment for user: {}", account_id))
        }
        }
        
        
    pub fn get_user_balance(&self, user_hash: String) -> Option<Decimal> {
        self.users.iter().find(|(u, _)| u == &user_hash).map(|(_, balance)| balance.clone())
    }

            
    pub fn place_bet(&mut self, user_hash: String, outcome: String, bet_amount: Decimal) -> bool {
        // Check if the user has sufficient balance
        if let Some((_, balance)) = self.users.iter().find(|(u, _)| u == &user_hash) {
            if *balance < bet_amount {
                return false; // Insufficient funds
            }
        } else {
            return false; // User not found
        }

        let taken_bucket = self.xrd_vault.take(bet_amount); // Here, 'take' should be returning a bucket of the specified amount.
        
        if let Some(index) = self.outcomes.iter().position(|o| o == &outcome) {
            let outcome_token = &mut self.outcome_tokens[index];
            outcome_token.put(taken_bucket);
            self.total_staked += bet_amount;
            return true;
        } else {
            self.xrd_vault.put(taken_bucket);  // Return the taken bucket if the outcome doesn't exist
            return false;
        }
    }


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
