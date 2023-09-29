use scrypto::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::TestRunnerBuilder;

#[test]
fn test_instantiate_prediction_market() -> Result<(), RuntimeError> {
    // Set up environment.
    let mut test_runner = TestRunnerBuilder::new().build();

    // Create an account
    let (public_key, _private_key, _account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Instantiate the MarketManager via a Manifest
    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "MarketManager",
            "new",
            manifest_args!(),
        )
        .build();
    let instantiate_receipt = test_runner.execute_manifest_ignoring_fee(instantiate_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    let market_manager_component = instantiate_receipt.expect_commit(true).new_component_addresses()[0];

    // Define your market parameters
    let market_id = String::from("market1");
    let outcomes_str = String::from("outcome1,outcome2");
    let odds_str = String::from("1.5,2.5");

    // Act: Instantiate the prediction market via a Manifest
    let act_manifest = ManifestBuilder::new()
        .call_method(
            market_manager_component,
            "instantiate_prediction_market",
            manifest_args!(market_id.clone(), outcomes_str.clone(), odds_str.clone()),
        )
        .build();
    let act_receipt = test_runner.execute_manifest_ignoring_fee(act_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    act_receipt.expect_commit_success();

    Ok(())
}

#[test]
fn test_retrieve_prediction_market() -> Result<(), RuntimeError> {
    // Set up environment.
    let mut test_runner = TestRunnerBuilder::new().build();

    // Create an account
    let (public_key, _private_key, _account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Instantiate the MarketManager via a Manifest
    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "MarketManager",
            "new",
            manifest_args!(),
        )
        .build();
    let instantiate_receipt = test_runner.execute_manifest_ignoring_fee(instantiate_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    let market_manager_component = instantiate_receipt.expect_commit(true).new_component_addresses()[0];

    // Define your market parameters
    let market_id = String::from("market1");
    let outcomes_str = String::from("outcome1,outcome2");
    let odds_str = String::from("1.5,2.5");

    // Instantiate the prediction market (as it's required for retrieval)
    let instantiation_manifest = ManifestBuilder::new()
        .call_method(
            market_manager_component,
            "instantiate_prediction_market",
            manifest_args!(market_id.clone(), outcomes_str.clone(), odds_str.clone()),
        )
        .build();
    test_runner.execute_manifest_ignoring_fee(instantiation_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]).expect_commit_success();

    // Act: Retrieve the market using the `get_market` function
    let get_market_manifest = ManifestBuilder::new()
        .call_method(
            market_manager_component,
            "get_market",
            manifest_args!(market_id.clone()),
        )
        .build();

    let get_market_receipt = test_runner.execute_manifest_ignoring_fee(get_market_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    get_market_receipt.expect_commit_success();  // Assert that the get_market operation was successful

    Ok(())
}

#[test]
fn test_list_all_markets() -> Result<(), RuntimeError> {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish(this_package!());

    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "MarketManager",
            "new",
            manifest_args!(),
        )
        .build();
    let instantiate_receipt = test_runner.execute_manifest_ignoring_fee(instantiate_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    let market_manager_component = instantiate_receipt.expect_commit(true).new_component_addresses()[0];

    let market_ids = vec!["market1", "market2", "market3"];
    for market_id in &market_ids {
        let act_manifest = ManifestBuilder::new()
            .call_method(
                market_manager_component,
                "instantiate_prediction_market",
                manifest_args!(market_id.to_string(), "outcome1,outcome2", "1.5,2.5"),
            )
            .build();
        let act_receipt = test_runner.execute_manifest_ignoring_fee(act_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
        act_receipt.expect_commit_success();
    }

    // Act: Call `list_all_markets` method
    let list_markets_manifest = ManifestBuilder::new()
        .call_method(
            market_manager_component,
            "list_all_markets",
            manifest_args!(),
        )
        .build();

    let list_markets_receipt = test_runner.execute_manifest_ignoring_fee(list_markets_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    list_markets_receipt.expect_commit_success();

    // Extract the list of market ids from the receipt
    let mut returned_market_ids: Vec<String> = list_markets_receipt.expect_commit_success().output(1);
    let mut expected_market_ids = market_ids.clone();

    returned_market_ids.sort();
    expected_market_ids.sort();

    assert_eq!(expected_market_ids, returned_market_ids);

    Ok(())
}
