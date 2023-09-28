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


        // Add a new method for users to claim their rewards from their vaults.
        pub fn claim_reward(&mut self, user_hash: String) -> Option<Bucket> {
            self.user_vaults.get_mut(&user_hash).map(|vault| vault.take_all())
            }
        }
    }
    }
