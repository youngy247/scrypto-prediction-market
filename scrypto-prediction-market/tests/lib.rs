use scrypto::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::TestRunnerBuilder;

#[test]
fn test_instantiate_prediction_market() -> Result<(), RuntimeError> {
    // Set up environment.
    let mut test_runner = TestRunnerBuilder::new().build();

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Define outcomes and odds
    let outcomes_str = "outcome1,outcome2".to_string();
    let odds_str = "2,3".to_string();

    // Instantiate the PredictionMarket via a Manifest
    let manifest1 = ManifestBuilder::new()
        .call_function(
            package_address,
            "PredictionMarket",
            "instantiate_prediction_market",
            manifest_args!(outcomes_str, odds_str),
        )
        .call_method(
          account_component,
          "deposit_batch",
          manifest_args!(ManifestExpression::EntireWorktop),
      )
        .build();
    
        let receipt1 = test_runner.execute_manifest_ignoring_fee(
          manifest1,
          vec![NonFungibleGlobalId::from_public_key(&public_key)],
      );
      println!("{:?}\n", receipt1);
      receipt1.expect_commit_success();
    
    // ... [If needed to use the badge later]
    // let use_badge_manifest = ManifestBuilder::new()
    //     .create_proof_from_account_of_amount(_account_component, admin_badge, dec!("1"))
    //     .call_method(prediction_market_component, "some_admin_method", manifest_args!())
    //     .build();
    // let use_badge_receipt = test_runner.execute_manifest_ignoring_fee(
    //     use_badge_manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );
    // use_badge_receipt.expect_commit_success();
    
    Ok(())
}
