use scrypto::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::TestRunnerBuilder;

#[test]
fn test_instantiate_and_get_market() -> Result<(), RuntimeError> {
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
            manifest_args!(market_id.clone(), outcomes_str, odds_str),
        )
        .build();
    let act_receipt = test_runner.execute_manifest_ignoring_fee(act_manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    act_receipt.expect_commit_success();

    // Assert: Retrieve the market and verify
    // This part will depend on the exact implementation of your get_market function and how you can retrieve the result.
    // But the main idea is to use another manifest to call the `get_market` function, and then check the result.
  
    Ok(())
}

