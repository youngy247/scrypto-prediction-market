use scrypto::prelude::*;

#[blueprint]
mod prediction_market {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            resolve_market => restrict_to: [admin]; 
            resolve_market_as_void => restrict_to: [admin];
            lock_market => restrict_to: [admin];
            claim_reward => PUBLIC;
            deposit_to_xrd_vault => PUBLIC;
            list_outcomes => PUBLIC;
            get_total_staked => PUBLIC;
            get_outcome_balance => PUBLIC;
            place_bet => PUBLIC;
            get_xrd_vault_balance => PUBLIC;
            get_market_details => PUBLIC;
        }
    }
    
    pub struct PredictionMarket {
        title: String,
        min_bet: Decimal,
        max_bet: Decimal,
        outcome_tokens: Vec<Vault>,
        outcomes: Vec<String>,
        odds: Vec<Decimal>,   
        total_staked: Decimal,
        bets: HashMap<String, Vec<(String, Decimal)>>,
        xrd_vault: Vault,
        user_vaults: HashMap<String, Vault>,
        market_resolved: bool,
        market_locked: bool,
    }

    impl PredictionMarket {

      //1. Initialization and Setup:
        pub fn instantiate_prediction_market(title: String, outcomes_str: String, odds_str: String, min_bet: Decimal, 
          max_bet: Decimal
  ) -> (Global<PredictionMarket>, FungibleBucket) {

            let outcomes: Vec<String> = outcomes_str.split(',').map(|s| s.trim().to_string()).collect();
            // Validate Uniqueness of Outcomes
            let unique_outcomes: HashSet<&str> = outcomes_str.split(',').collect();
            assert_eq!(
                unique_outcomes.len(),
                outcomes.len(),
                "Duplicate outcomes provided."
            );


            let odds: Vec<Decimal> = odds_str.split(',')
                .map(|s| Decimal::from_str(s.trim()).expect("Failed to parse odds as Decimal"))
                .collect();

              // Validate Odds
                for odd in &odds {
                  assert!(
                      *odd > Decimal::from(1),
                      "Odds must be greater than 1. Provided: {}",
                      odd
                  );
              }
        
              assert_eq!(
                outcomes.len(),
                odds.len(),
                "The number of odds provided does not match the number of outcomes."
            );

              // Validate Min and Max Bet
              assert!(
                min_bet >= Decimal::from(5),
                "Minimum bet must be atleast 5. Provided: {}",
                min_bet
              );

              assert!(
                max_bet > min_bet,
                "Maximum bet must be greater than the minimum bet. Provided: Max bet: {}, Min bet: {}",
                max_bet, min_bet
              );

        
            let mut outcome_tokens = Vec::new();
            for _ in &outcomes {
                outcome_tokens.push(Vault::new(XRD)); // Create a new XRD vault for each outcome
            }

            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None) // #1
            .metadata(metadata!(init{"name"=>"admin badge", locked;}))
            .divisibility(DIVISIBILITY_NONE)
            .mint_initial_supply(1);

            
            let component = Self {
                title,
                min_bet,
                max_bet,
                outcome_tokens,
                outcomes,
                odds,  
                total_staked: Decimal::from(0),
                bets: HashMap::new(),
                xrd_vault: Vault::new(XRD),
                user_vaults: HashMap::new(),
                market_resolved: false,
                market_locked: false,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(admin_badge.resource_address()));
            ))
            .globalize();

            // Return the component address and the owner_badge
            (
                component,
                admin_badge
            )
        }

        pub fn deposit_to_xrd_vault(&mut self, deposit: Bucket) {

          self.xrd_vault.put(deposit);
      }

      pub fn get_xrd_vault_balance(&self) -> Decimal {
        Decimal::from(self.xrd_vault.amount())
    }

    //2. Market Management:

        // Locks the market to prevent further bets.
          pub fn lock_market(&mut self) {
            self.market_locked = true;
          }

          pub fn resolve_market(&mut self, winning_outcome: u32) -> Result<Vec<(String, Decimal)>, String> {
            // Check if the winning_outcome is within the valid range of outcomes.
            assert!((winning_outcome as usize) < self.outcome_tokens.len(), "Winning outcome is out of bounds.");
            // Ensure the market hasn't been resolved before.
            self.ensure_market_not_resolved();
        
            println!("Resolving market for winning outcome: {}", winning_outcome);
        
            // Initialize an empty vector to store the rewards for each user.
            let mut rewards = Vec::new();
        
            // Iterate through each outcome's vault to process losing vaults.
            for (index, outcome_vault) in self.outcome_tokens.iter_mut().enumerate() {
                if index == winning_outcome as usize {
                    continue; // Skip the winning vault.
                }
        
                // Take all tokens from the losing vault.
                let tokens = outcome_vault.take_all();
                println!("Tokens taken from losing vault {}: {:?}", index, tokens);
        
                // Transfer tokens from losing vaults to the xrd_vault.
                self.xrd_vault.put(tokens);
            }
        
            println!("Total amount in xrd_vault after transferring from losing vaults: {}", self.xrd_vault.amount());
        
            // Get the total amount staked for the winning outcome.
            let winning_outcome_str = &self.outcomes[winning_outcome as usize];
            let total_winning_staked = self.bets.get(winning_outcome_str)
            .map_or(Decimal::from(0), |bets| bets.iter().fold(Decimal::from(0), |acc, (_, amt)| acc + *amt));
  
        
            println!("Total amount staked for the winning outcome {}: {}", winning_outcome, total_winning_staked);
        
            // If there are bets for the winning outcome, process them.
            if let Some(winning_bets) = self.bets.get(winning_outcome_str) {
                for (user, bet_amt) in winning_bets {
                    // Calculate the user's proportion of the total staked amount for the winning outcome.
                    let user_proportion = *bet_amt / total_winning_staked;
        
                    println!("User {} proportion of total winning stake: {}", user, user_proportion);
        
                    // Calculate the reward based on the odds and the user's proportion of the winning stake.
                    let user_reward = *bet_amt * self.odds[winning_outcome as usize];
        
                    println!("Calculated reward for user {}: {}", user, user_reward);
        
                    // Store the user and their reward in the rewards vector.
                    rewards.push((user.clone(), user_reward));
        
                    // Extract the reward from the xrd_vault.
                    let reward_bucket = self.xrd_vault.take(user_reward);
        
                    // Transfer the reward to the user's vault.
                    if let Some(user_vault) = self.user_vaults.get_mut(user) {
                        user_vault.put(reward_bucket);
                    }
                }
            }
        
            // Reset the total_staked amount to 0 as the market is now resolved.
            self.total_staked = Decimal::from(0);
            println!("Reset total staked to 0.");
        
            // Mark the market as resolved to prevent further interactions.
            self.market_resolved = true;
        
            // Return the rewards vector as the result of the function.
            Ok(rewards)
        }
  
        pub fn resolve_market_as_void(&mut self) -> Result<(), String> {
          // Ensure the market hasn't been resolved before.
          self.ensure_market_not_resolved();
      
          // Iterate through each outcome's vault.
          for outcome_vault in &mut self.outcome_tokens {
              // Take all tokens from the outcome vault.
              let tokens = outcome_vault.take_all();
      
              // Transfer tokens from outcome vaults to the xrd_vault.
              self.xrd_vault.put(tokens);
          }
      
          // Iterate over all the user bets and refund them.
          for (_, outcome_bets) in &self.bets {
              for (user, bet_amt) in outcome_bets {
                  // Extract the refund amount from the xrd_vault.
                  let refund_bucket = self.xrd_vault.take(*bet_amt);
      
                  // Transfer the refund to the user's vault.
                  if let Some(user_vault) = self.user_vaults.get_mut(user) {
                      user_vault.put(refund_bucket);
                  }
              }
          }
      
          // Reset the total_staked amount to 0 as the market is now resolved.
          self.total_staked = Decimal::from(0);
      
          // Mark the market as resolved to prevent further interactions.
          self.market_resolved = true;
      
          // Return Ok to indicate the market was successfully resolved as void.
          Ok(())
      }

      // 3. Betting and Claiming Rewards:
      pub fn place_bet(&mut self, user_hash: String, outcome: String, payment: Bucket) {
        // Ensure the market hasn't been resolved before.
        self.ensure_market_not_resolved();
        
        // Validate the bet.
        self.validate_bet(&payment);
    
        // Get the outcome's position.
        let outcome_position = self.get_outcome_position(&outcome);
    
        // Ensure user vault exists.
        self.ensure_user_vault_exists(user_hash.clone());
    
        // Process the bet.
        self.process_bet(user_hash, outcome_position, payment.amount(), payment);
    }

    pub fn claim_reward(&mut self, user_hash: String) -> Option<Bucket> {
      // Attempt to get a mutable reference to the user's vault using the provided user_hash.
      // The map method is used to access the user's vault if it exists.
      self.user_vaults.get_mut(&user_hash).map(|vault| {
          // If the user's vault exists, take all tokens from the vault as the reward.
          // The take_all method returns a Bucket containing all the tokens from the vault.
          vault.take_all()
      })
      // If the user's vault does not exist, the map method will return None,
      // and so will the claim_reward function.
  }

        // 4. Getters:
        
        pub fn list_outcomes(&self) -> Vec<String> {
            self.outcomes.clone()
        }

        pub fn get_total_staked(&self) -> Decimal {
            self.total_staked.clone()
        }

        pub fn get_market_details(&self) -> (String, Vec<String>, Vec<Decimal>, Decimal) {
          (self.title.clone(), self.outcomes.clone(), self.odds.clone(), self.total_staked.clone())
      }
      

        pub fn get_outcome_balance(&self, outcome: String) -> Decimal {
            assert!(self.outcomes.contains(&outcome), "Outcome does not exist.");
            
            let index = self.outcomes.iter().position(|o| o == &outcome).expect("Outcome not found.");
            Decimal::from(self.outcome_tokens[index].amount())
        }

        // 5. Helpers:
        
        fn ensure_market_not_resolved(&self) {
          assert!(!self.market_resolved, "Market '{}' has already been resolved.", self.title);
      }

        fn ensure_user_vault_exists(&mut self, user_hash: String) {
          // Check if a vault exists for the user, if not, create a new one.
          if !self.user_vaults.contains_key(&user_hash) {
              self.user_vaults.insert(user_hash.clone(), Vault::new(XRD));
          }
      }

        // Validate the bet using assertions.
        fn validate_bet(&self, payment: &Bucket) {
          // Assert the market is not locked.
          assert!(
              !self.market_locked, 
              "Market '{}' is locked. No more bets can be placed.", 
              self.title
          );
          
          let bet_amount = payment.amount();
          
          assert!(
              bet_amount >= self.min_bet,
              "Bet amount {} is below the minimum allowed of {}.", 
              bet_amount, self.min_bet
          );
          
          assert!(
              bet_amount <= self.max_bet, 
              "Bet amount {} exceeds the maximum allowed of {}.", 
              bet_amount, self.max_bet
          );

          assert!(
              bet_amount > Decimal::from(0),
              "Invalid bet amount."
          );
        }

        // Process the bet.
        fn process_bet(&mut self, user_hash: String, outcome_position: usize, bet_amount: Decimal, payment: Bucket) {
          // Get a mutable reference to the vault associated with the outcome.
          let outcome_token = &mut self.outcome_tokens[outcome_position];
          // Deposit the payment into the outcome's vault.
          outcome_token.put(payment);
          // Update the total amount staked in the market.
          self.total_staked += bet_amount;
          // Record the bet.
          let outcome = &self.outcomes[outcome_position];
          let outcome_bets = self.bets.entry(outcome.clone()).or_insert_with(Vec::new);
          outcome_bets.push((user_hash, bet_amount));
        }

        // Get outcome position using assertion
      fn get_outcome_position(&self, outcome: &String) -> usize {
        self.outcomes.iter().position(|o| o == outcome)
            .expect(&format!("Outcome '{}' does not exist. The available outcomes are: {:?}", outcome, self.outcomes))
      }
    

    }        
}
