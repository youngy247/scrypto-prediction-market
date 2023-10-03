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
    let title= "title".to_string();
    let outcomes_str = "outcome1,outcome2".to_string();
    let odds_str = "2,3".to_string();
    let min_bet = dec!("5");
    let max_bet = dec!("100");
    // Instantiate the PredictionMarket via a Manifest
    let manifest1 = ManifestBuilder::new()
        .call_function(
            package_address,
            "PredictionMarket",
            "instantiate_prediction_market",
            manifest_args!(title, outcomes_str, odds_str, min_bet, max_bet),
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

#[test]
fn test_list_outcomes() -> Result<(), RuntimeError> {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish(this_package!());

    // Define outcomes and odds
    let title = "title".to_string();
    let outcomes_str = "outcome1,outcome2".to_string();
    let odds_str = "2,3".to_string();
    let min_bet = dec!("5");
    let max_bet = dec!("100");

    // Instantiate the PredictionMarket
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "PredictionMarket",
            "instantiate_prediction_market",
            manifest_args!(
                title.clone(),
                outcomes_str.clone(),
                odds_str.clone(),
                min_bet.clone(),
                max_bet.clone()
            ),
        )
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    let market_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Call the list_outcomes method
    let list_outcomes_manifest = ManifestBuilder::new()
        .call_method(
            market_address,
            "list_outcomes",
            manifest_args!(),
        )
        .build();

    let list_outcomes_receipt = test_runner.execute_manifest_ignoring_fee(list_outcomes_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    list_outcomes_receipt.expect_commit_success();

    // Extract the list of outcomes from the receipt
    let outcomes: Vec<String> = list_outcomes_receipt.expect_commit_success().output(1);

    // Assert the outcomes
    assert_eq!(outcomes, outcomes_str.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>());

    Ok(())
}
