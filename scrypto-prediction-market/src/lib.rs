/*
---------------------------------------------------
DEV NOTE: PREDICTION MARKET IN SCRYPTO
---------------------------------------------------

OVERVIEW:
This blueprint represents a prediction market on Scrypto where users can place bets on potential outcomes, and market admins can manage the market's state.

FUNCTIONALITY HIGHLIGHTS:
1.  Events are emitted for several major actions 
    (e.g., when a market is resolved, when a bet is placed). 
    These events can be monitored by a front-end application 
    to provide real-time feedback to users.
2.  Methods are organized into 5 main sections for clarity:
        - Initialization and Setup
        - Market Management (Admin only)
        - Betting and Claiming Rewards (Users only)
        - Getters (Methods to fetch specific data)
        - Helper Functions (Internal utility functions)

SPECIFIC FUNCTION AND METHOD OVERVIEWS:
1.  Initialization and Setup:
        - `instantiate_prediction_market`: Set up the market with given parameters.
        - `deposit_to_xrd_vault`: Allow deposits to the market's XRD vault.
        - `get_xrd_vault_balance`: Fetch the current balance of the XRD vault.

2.  Market Management (Admin-only):
        - `lock_market`: Prevent further bets on this market.
        - `withdraw_from_vault`: Admin can withdraw a specified amount from the xrd_vault.
        - `admin_claim`: Admin can claim tokens from the admin_vault.
        - `resolve_market`: Determine the winning outcome and distribute rewards.
        - `resolve_market_as_void`: Void the market and refund all bets.

3.  Betting and Claiming Rewards (Users only):
        - `place_bet`: A user places a bet on an outcome. Validation ensures the bet is valid, and the bet amount is staked on the chosen outcome.
        - `claim_reward`: A user claims their reward. If the user has a reward in their vault, it's returned to them.

4.  Getters:
        - `list_outcomes`: List all possible outcomes in the market.
        - `get_total_staked`: Get the total amount staked in the market.
        - `get_market_details`: Fetch the market's details, including title, possible outcomes, odds, and total staked amount.
        - `get_outcome_balance`: Get the total amount staked for a specific outcome.

5.  Helper Functions (Internal utility functions):
        - `ensure_market_not_resolved`: Ensure the market hasn't been resolved before proceeding.
        - `ensure_user_vault_exists`: Ensure a user vault exists or create one if it doesn't.
        - `validate_bet`: Validate the provided bet ensuring the amount is within limits and the market isn't locked.
        - `get_outcome_position`: Get the index position of a specified outcome in the market.
        - `reset_and_resolve_market`: Reset the total staked amount and mark the market as resolved.

 */

use scrypto::prelude::*;

/// About the `market_id` field in the events below:
/// - The `market_id` serves as the identifier for the market.
/// - Currently, it's set using the title of the market.
/// - For unique identification, especially in cases with multiple instances of the same market,
///   consider transitioning to a UUID.

/// Represents an event that gets emitted when a market is resolved.
/// This means that the outcome of the market is determined.
#[derive(ScryptoSbor, ScryptoEvent)]
struct MarketResolvedEvent {
    market_id: String,  
    winning_outcome: u32, // The index representing the winning outcome of the market.
}

/// Represents an event when a market is resolved as void.
/// Can occur if a market has an ambiguous or indeterminate outcome.
#[derive(ScryptoSbor, ScryptoEvent)]
struct MarketResolvedAsVoidEvent {
    market_id: String,
}

/// Event that indicates when a market is locked, preventing further bets.
#[derive(ScryptoSbor, ScryptoEvent)]
struct MarketLockedEvent {
    market_id: String,
}

/// Event emitted when a user places a bet on a specific market outcome.
#[derive(ScryptoSbor, ScryptoEvent)]
struct BetPlacedEvent {
    market_id: String,
    user_hash: String,  // Unique identifier for the user placing the bet.
    outcome: String,    // Chosen outcome the user is betting on.
    amount: Decimal,    // Amount of XRD the user is betting.
}

/// Event emitted when a user claims their reward after a market's resolution.
#[derive(ScryptoSbor, ScryptoEvent)]
struct ClaimRewardEvent {
    market_id: String,
    user_hash: String,  // Unique identifier for the user claiming the reward.
    reward: Decimal,    // Amount of the XRD reward being claimed.
}


#[blueprint]
#[events(MarketResolvedEvent, MarketLockedEvent, BetPlacedEvent, MarketResolvedAsVoidEvent, ClaimRewardEvent)]
mod prediction_market {
    
    // Method authentication setup. 
    // Specifies roles and access permissions for different methods.
    enable_method_auth! {
        
        // Roles and their updatable conditions.
        roles {
            // The `admin` role has no updatable conditions, meaning once set, it remains fixed.
            admin => updatable_by: [];
        },
        
        // Specify which methods can be accessed by which roles.
        methods {
            // Only the `admin` can resolve, lock, and resolve the market as void.
            resolve_market => restrict_to: [admin]; 
            resolve_market_as_void => restrict_to: [admin];
            lock_market => restrict_to: [admin];
            
            // These methods can be accessed by any user.
            withdraw_from_vault => PUBLIC;
            admin_claim => PUBLIC;
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
    
    // Primary structure for the prediction market.
    pub struct PredictionMarket {
        // Title or name of the market.
        title: String,
        
        // Minimum and maximum bet amounts allowed.
        min_bet: Decimal,
        max_bet: Decimal,
        
        // Vaults associated with each potential market outcome.
        outcome_tokens: Vec<Vault>,
        
        // Possible outcomes in the market.
        outcomes: Vec<String>,
        
        // Odds associated with each outcome.
        odds: Vec<Decimal>,   
        
        // Total amount staked in the market.
        total_staked: Decimal,
        
        // Records of all bets placed, categorized by outcome.
        // Each entry consists of the user's hash and the amount they bet.
        bets: HashMap<String, Vec<(String, Decimal)>>,
        
        // Vault for the XRD token (potentially the primary currency of the system).
        xrd_vault: Vault,
        
        // Vault for the admin.
        admin_vault: Vault,
        
        // Vaults for individual users, mapped by user hash.
        user_vaults: HashMap<String, Vault>,
        
        // Flag to indicate if the market has been resolved.
        market_resolved: bool,
        
        // Flag to indicate if the market is locked (no more betting allowed).
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
                admin_vault: Vault::new(XRD),
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

        //2. Market Management - Admin only:

        // Locks the market to prevent further bets.
        pub fn lock_market(&mut self) {
            self.market_locked = true;

            Runtime::emit_event(MarketLockedEvent {
                market_id: self.title.clone(),
            });
        }

        pub fn withdraw_from_vault(&mut self, amount: Decimal) {
            // Ensure the xrd_vault has enough funds to fulfill the withdrawal request.
            assert!(self.xrd_vault.amount() >= amount, "Insufficient funds in xrd_vault.");
        
            // Take the specified amount from the xrd_vault.
            let withdrawal_bucket = self.xrd_vault.take(amount);
            self.admin_vault.put(withdrawal_bucket);
        }

        pub fn admin_claim(&mut self) -> Option<Bucket> {
            // Take all tokens from the admin_vault.
            let bucket = self.admin_vault.take_all();

            // Assert that the bucket is not empty.
            assert!(!bucket.is_empty(), "Bucket is empty");

            Some(bucket)
        }


        pub fn resolve_market(&mut self, winning_outcome: u32) -> Result<Vec<(String, Decimal)>, String> {
            // Ensure the market hasn't been resolved before.
            self.ensure_market_not_resolved();

            println!("Resolving market for winning outcome: {}", winning_outcome);
            // Check if the winning_outcome is within the valid range of outcomes.
            assert!((winning_outcome as usize) < self.outcome_tokens.len(), "Winning outcome is out of bounds.");
        
        
        
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
        
            // Reset the total_staked amount to 0 and mark the market as resolved to prevent further interactions.
            self.reset_and_resolve_market();

            // Emit the MarketResolvedEvent right after the market is resolved.
            Runtime::emit_event(MarketResolvedEvent {
                market_id: self.title.clone(),
                winning_outcome,
            });


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
    
            // Reset the total_staked amount to 0 and mark the market as resolved to prevent further interactions.
            self.reset_and_resolve_market();

            // Emit the MarketResolvedAsVoidEvent right after the market is resolved as void.
            Runtime::emit_event(MarketResolvedAsVoidEvent {
                market_id: self.title.clone(),
            });

    
            // Return Ok to indicate the market was successfully resolved as void.
            Ok(())
        }

      // 3. Betting and Claiming Rewards - Users only:
        pub fn place_bet(&mut self, user_hash: String, outcome: String, payment: Bucket) {
            // Ensure the market hasn't been resolved before.
            self.ensure_market_not_resolved();
            
            // Validate the bet.
            self.validate_bet(&payment);
        
            // Get the outcome's position.
            let outcome_position = self.get_outcome_position(&outcome);
        
            // Ensure user vault exists.
            self.ensure_user_vault_exists(user_hash.clone());
        
            // Extract payment amount before moving `payment`
            let payment_amount = payment.amount();

            // Get a mutable reference to the vault associated with the outcome.
            let outcome_token = &mut self.outcome_tokens[outcome_position];
            // Deposit the payment into the outcome's vault.
            outcome_token.put(payment);
            // Update the total amount staked in the market.
            self.total_staked += payment_amount;
            // Record the bet.
            let outcome_clone = self.outcomes[outcome_position].clone();
            let outcome_bets = self.bets.entry(outcome_clone).or_insert_with(Vec::new);
            outcome_bets.push((user_hash.clone(), payment_amount));

            // Emit the BetPlacedEvent.
            Runtime::emit_event(BetPlacedEvent {
                market_id: self.title.clone(),
                user_hash,
                outcome,
                amount: payment_amount,
            });

    }

    pub fn claim_reward(&mut self, user_hash: String) -> Option<Bucket> {
        // Attempt to get a mutable reference to the user's vault using the provided user_hash.
        if let Some(vault) = self.user_vaults.get_mut(&user_hash) {
            // If the user's vault exists, take all tokens from the vault as the reward.
            let bucket = vault.take_all();
            
            // Assert that the bucket is not empty.
            assert!(!bucket.is_empty(), "Bucket is empty");

            // Emit an event to indicate successful reward claim.
            Runtime::emit_event(ClaimRewardEvent {
                market_id: self.title.clone(),
                user_hash: user_hash.clone(),
                reward: bucket.amount(),
            });
            
            Some(bucket)
        

            } else {
            // If the user's vault does not exist, return None.
            None
        }
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

        // Get outcome position using assertion
        fn get_outcome_position(&self, outcome: &String) -> usize {
            self.outcomes.iter().position(|o| o == outcome)
            .expect(&format!("Outcome '{}' does not exist. The available outcomes are: {:?}", outcome, self.outcomes))
        } 

        fn reset_and_resolve_market(&mut self) {
        self.total_staked = Decimal::from(0);
        self.market_resolved = true;
        }

    }        
}
