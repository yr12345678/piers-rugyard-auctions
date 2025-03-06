use crate::unit_tests::helpers::*;
use piers_rugyard::types::*;
use scrypto_test::prelude::*;

#[test]
fn can_instantiate() -> Result<(), RuntimeError> {
    let (_, _, _) = create_prepared_test_environment()?;

    Ok(())
}

#[test]
fn can_mint_nft() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let available_nfts_count = get_state_available_nfts_list(&mut env, component)?.len();
    let nft_manager = ResourceManager(get_state_resource_address(&mut env, component)?);
    let nft_count = nft_manager
        .total_supply(&mut env)
        .expect("Could not get supply!")
        .expect("Could not get supply!");

    // Act
    mint_nft(&mut env, component)?;

    // Assert
    let new_available_nfts_count = get_state_available_nfts_list(&mut env, component)?.len();
    assert!(
        new_available_nfts_count - available_nfts_count == 1,
        "Available NFT count did not increase by 1"
    );

    let new_nft_count = nft_manager
        .total_supply(&mut env)
        .expect("Could not get supply!")
        .expect("Could not get supply!");
    assert!(
        new_nft_count - nft_count == dec!(1),
        "NFT supply did not increase by 1"
    );

    Ok(())
}

#[test]
fn cannot_mint_nft_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, component, _owner_resource) = create_prepared_test_environment()?;

    // Act
    let result = mint_nft(&mut env, component);

    // Assert
    assert!(result.is_err(), "Could mint NFT without owner badge!");

    Ok(())
}

#[test]
fn can_switch_auction_status() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let active = get_state_active(&mut env, component)?;

    // Act
    component.flip_status(&mut env)?;

    // Assert
    assert!(
        get_state_active(&mut env, component)? != active,
        "Could not switch auction status"
    );

    Ok(())
}

#[test]
fn cannot_switch_auction_status_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    // Act
    let result = component.flip_status(&mut env);

    // Assert
    assert!(
        result.is_err(),
        "Could switch auction status without owner badge"
    );

    Ok(())
}

#[test]
fn can_delete_nft() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let available_nfts_count = get_state_available_nfts_list(&mut env, component)?.len();
    let nft_manager = ResourceManager(get_state_resource_address(&mut env, component)?);
    let nft_count = nft_manager
        .total_supply(&mut env)
        .expect("Could not get supply!")
        .expect("Could not get supply!");

    let local_id_to_delete = NonFungibleLocalId::integer(1);

    // Act
    component.delete_nft(local_id_to_delete.clone(), &mut env)?;

    // Assert
    let new_available_nfts = get_state_available_nfts_list(&mut env, component)?;
    assert!(
        available_nfts_count - new_available_nfts.len() == 1,
        "Available NFT count did not decrease by 1"
    );

    let new_nft_count = nft_manager
        .total_supply(&mut env)
        .expect("Could not get supply!")
        .expect("Could not get supply!");
    assert!(
        nft_count - new_nft_count == dec!(1),
        "NFT supply did not increase by 1"
    );

    assert!(
        !new_available_nfts.contains(&local_id_to_delete),
        "NFT is still in the list of available NFTs"
    );

    let nft_data_result =
        nft_manager.get_non_fungible_data::<_, _, NFT>(local_id_to_delete, &mut env);
    assert!(nft_data_result.is_err(), "Was able to get NFT data for id");

    Ok(())
}

#[test]
fn cannot_delete_nft_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;
    let local_id_to_delete = NonFungibleLocalId::integer(1);

    // Act
    let result = component.delete_nft(local_id_to_delete.clone(), &mut env);

    // Assert
    assert!(result.is_err(), "Could delete NFT without owner badge!");

    Ok(())
}

#[test]
fn can_update_pool_address() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let old_pool_address = get_state_pool_address(&mut env, component)?;

    let (_oci_pool, oci_pool_address, _early_resource_address) = instantiate_oci_pool(&mut env)?;

    // Act
    component.update_pool_address(oci_pool_address, &mut env)?;

    // Assert
    let new_pool_address = get_state_pool_address(&mut env, component)?;

    assert!(
        new_pool_address != old_pool_address,
        "New and old pool address are the same!"
    );
    assert!(
        new_pool_address == oci_pool_address,
        "Pool address was not updated to new one!"
    );

    Ok(())
}

#[test]
fn cannot_update_pool_address_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let (_oci_pool, oci_pool_address, _early_resource_address) = instantiate_oci_pool(&mut env)?;

    // Act
    let result = component.update_pool_address(oci_pool_address, &mut env);

    // Assert
    assert!(
        result.is_err(),
        "Could change pool address without owner badge!"
    );

    Ok(())
}

#[test]
fn can_update_auction_duration() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let old_auction_duration = get_state_auction_duration(&mut env, component)?;

    // Act
    component.update_auction_duration(old_auction_duration + 5, &mut env)?;

    // Assert
    let new_auction_duration = get_state_auction_duration(&mut env, component)?;

    assert!(
        new_auction_duration == old_auction_duration + 5,
        "Auction duration has not changed!"
    );

    Ok(())
}

#[test]
fn cannot_update_auction_duration_shorter_than_auction_buffer() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let auction_buffer = get_state_auction_buffer(&mut env, component)?;

    // Act
    let result = component.update_auction_duration(auction_buffer - 1, &mut env);

    // Assert
    assert!(
        result.is_err(),
        "Was able to set auction duration lower than auction buffer!"
    );

    Ok(())
}

#[test]
fn cannot_update_auction_duration_to_zero() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    // Act
    let result = component.update_auction_duration(0, &mut env);

    // Assert
    assert!(result.is_err(), "Was able to set auction duration to 0!");

    Ok(())
}

#[test]
fn cannot_update_auction_duration_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;
    let old_auction_duration = get_state_auction_duration(&mut env, component)?;

    // Act
    let result = component.update_auction_duration(old_auction_duration + 5, &mut env);

    // Assert
    assert!(
        result.is_err(),
        "Could change auction duration without owner badge!"
    );

    Ok(())
}

#[test]
fn can_update_auction_buffer() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let old_auction_buffer = get_state_auction_buffer(&mut env, component)?;

    // Act
    component.update_auction_buffer(old_auction_buffer - 1, &mut env)?;

    // Assert
    let new_auction_buffer = get_state_auction_buffer(&mut env, component)?;

    assert!(
        new_auction_buffer == old_auction_buffer - 1,
        "Auction buffer has not changed!"
    );

    Ok(())
}

#[test]
fn cannot_update_auction_buffer_longer_than_auction_duration() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let auction_duration = get_state_auction_duration(&mut env, component)?;

    // Act
    let result = component.update_auction_buffer(auction_duration + 1, &mut env);

    // Assert
    assert!(
        result.is_err(),
        "Could set auction buffer longer than auction duration!"
    );

    Ok(())
}

#[test]
fn cannot_update_auction_buffer_to_zero() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    // Act
    let result = component.update_auction_buffer(0, &mut env);

    // Assert
    assert!(result.is_err(), "Could set auction buffer to zero!");

    Ok(())
}

#[test]
fn cannot_update_auction_buffer_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let old_auction_buffer = get_state_auction_buffer(&mut env, component)?;

    // Act
    let result = component.update_auction_buffer(old_auction_buffer - 1, &mut env);

    // Assert
    assert!(
        result.is_err(),
        "Could change auction buffer without owner badge!"
    );

    Ok(())
}

#[test]
fn can_update_minimum_bid_increase() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let old_minimum_bid_increase = get_state_minimum_bid_increase(&mut env, component)?;

    // Act
    component.update_minimum_bid_increase(old_minimum_bid_increase + dec!(25), &mut env)?;

    // Assert
    let new_minimum_bid_increase = get_state_minimum_bid_increase(&mut env, component)?;

    assert!(
        new_minimum_bid_increase == old_minimum_bid_increase + dec!(25),
        "Minimum bid incease has not changed!"
    );

    Ok(())
}

#[test]
fn cannot_update_minimum_bid_increase_to_zero() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    // Act
    let result = component.update_minimum_bid_increase(dec!(0), &mut env);

    // Assert
    assert!(result.is_err(), "Minimum bid incease was changed to 0!");

    Ok(())
}

#[test]
fn cannot_update_minimum_bid_increase_without_owner() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let old_minimum_bid_increase = get_state_minimum_bid_increase(&mut env, component)?;

    // Act
    let result = component.update_minimum_bid_increase(old_minimum_bid_increase + 25, &mut env);

    // Assert
    assert!(
        result.is_err(),
        "Minimum bid increase was changed without an owner badge!"
    );

    Ok(())
}

#[test]
fn can_settle_auction_after_auction_ended() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;
    component.bid(xrd_bucket.into(), account, &mut env)?;

    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    env.set_current_time(
        current_auction
            .end_timestamp
            .add_minutes(1)
            .expect("Could not add time"),
    );

    // Act
    let result = component.settle_auction(account, &mut env);

    // Assert
    assert!(result.is_ok(), "Could not settle auction!");

    Ok(())
}

#[test]
fn can_settle_auction_after_auction_ended_with_no_new_nfts() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let owner_proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(owner_proof, &mut env)?;
    component.delete_nft(NonFungibleLocalId::integer(2), &mut env)?; // Only 1 NFT left after this

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;
    component.bid(xrd_bucket.into(), account, &mut env)?;

    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");

    env.set_current_time(
        current_auction
            .end_timestamp
            .add_minutes(1)
            .expect("Could not add time"),
    );

    // Act
    let result = component.settle_auction(account, &mut env);

    // Assert
    assert!(result.is_ok(), "Could not settle auction!");

    Ok(())
}

#[test]
fn cannot_settle_auction_before_auction_ended() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;
    component.bid(xrd_bucket.into(), account, &mut env)?;

    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    env.set_current_time(
        current_auction
            .end_timestamp
            .add_minutes(-1)
            .expect("Could not add time"),
    );

    // Act
    let result = component.settle_auction(account, &mut env);

    // Assert
    assert!(result.is_err(), "Could settle auction before endtime!");

    Ok(())
}

#[test]
fn cannot_settle_auction_without_bids() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;

    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    env.set_current_time(
        current_auction
            .end_timestamp
            .add_minutes(1)
            .expect("Could not add time"),
    );

    // Act
    let result = component.settle_auction(account, &mut env);

    // Assert
    assert!(result.is_err(), "Could settle auction without bids!");

    Ok(())
}

#[test]
fn can_withdraw_profits() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, owner_resource) = create_prepared_test_environment()?;

    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;
    component.bid(xrd_bucket.into(), account, &mut env)?;

    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    env.set_current_time(
        current_auction
            .end_timestamp
            .add_minutes(1)
            .expect("Could not add time"),
    );

    component.settle_auction(account, &mut env)?;

    // Act
    let profits = component.withdraw_profits(&mut env)?;

    // Assert
    assert!(
        profits.amount(&mut env)? > dec!(0),
        "Profits not higher than 0!"
    );

    Ok(())
}

#[test]
fn can_bid_active_auction() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;

    // Act
    let result = component.bid(xrd_bucket.into(), account, &mut env);

    // Assert
    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");

    assert!(result.is_ok(), "Could not bid on active auction");

    assert!(
        current_auction.highest_bid.unwrap() == dec!(1000),
        "Highest bid was not 1000"
    );

    Ok(())
}

#[test]
fn cannot_bid_below_minimum_bid_amount() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let minimum_bid_increase = get_state_minimum_bid_increase(&mut env, component)?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(
        XRD,
        minimum_bid_increase - 1, // Bid below the minimum increase
        Mock,
        &mut env,
    )?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;

    // Act
    let result = component.bid(xrd_bucket.into(), account, &mut env);

    // Assert
    assert!(result.is_err(), "Could bid below minimum bid increase");
    Ok(())
}

#[test]
fn cannot_bid_same_as_highest_bid() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket1 = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;
    let xrd_bucket2 = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account1 = create_account(&mut env, XRD)?;
    let account2 = create_account(&mut env, XRD)?;
    let account_proof1 = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    let account_proof2 = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof1, &mut env)?;
    LocalAuthZone::push(account_proof2, &mut env)?;

    component.start_new_auction(&mut env)?;

    // Act
    component.bid(xrd_bucket1.into(), account1, &mut env)?;
    let result = component.bid(xrd_bucket2.into(), account2, &mut env);

    // Assert
    assert!(result.is_err(), "Could bid the same as current highest bid");

    Ok(())
}

#[test]
fn cannot_bid_lower_than_highest_bid() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket1 = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;
    let xrd_bucket2 = BucketFactory::create_fungible_bucket(XRD, dec!(990), Mock, &mut env)?;

    let account1 = create_account(&mut env, XRD)?;
    let account2 = create_account(&mut env, XRD)?;
    let account_proof1 = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    let account_proof2 = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof1, &mut env)?;
    LocalAuthZone::push(account_proof2, &mut env)?;

    component.start_new_auction(&mut env)?;

    // Act
    component.bid(xrd_bucket1.into(), account1, &mut env)?;
    let result = component.bid(xrd_bucket2.into(), account2, &mut env);

    // Assert
    assert!(result.is_err(), "Could bid below current highest bid");

    Ok(())
}

#[test]
fn can_bid_ended_auction_without_bids() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;

    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    env.set_current_time(current_auction.end_timestamp.add_hours(1).unwrap()); // Move time past the end time

    // Act
    let result = component.bid(xrd_bucket.into(), account, &mut env);

    // Assert
    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");

    assert!(
        result.is_ok(),
        "Could not bid on ended auction without bids"
    );

    assert!(
        current_auction.highest_bid.unwrap() == dec!(1000),
        "Highest bid was not 1000"
    );

    Ok(())
}

#[test]
fn cannot_bid_after_auction_end_with_existing_bids() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket1 = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let xrd_bucket2 = BucketFactory::create_fungible_bucket(XRD, dec!(1500), Mock, &mut env)?;

    let account1 = create_account(&mut env, XRD)?;
    let account2 = create_account(&mut env, XRD)?;
    let account_proof1 = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    let account_proof2 = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof1, &mut env)?;
    LocalAuthZone::push(account_proof2, &mut env)?;

    component.start_new_auction(&mut env)?;

    // Act
    component.bid(xrd_bucket1.into(), account1, &mut env)?; // Bid 1
    let current_auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    env.set_current_time(current_auction.end_timestamp.add_hours(1).unwrap()); // Move time past the end time
    let result = component.bid(xrd_bucket2.into(), account2, &mut env); // Bid 2

    // // Assert
    assert!(
        result.is_err(),
        "Could bid on ended auction with active bids"
    );

    Ok(())
}

#[test]
fn active_auction_is_extended() -> Result<(), RuntimeError> {
    // Arrange
    let (mut env, mut component, _owner_resource) = create_prepared_test_environment()?;

    let xrd_bucket = BucketFactory::create_fungible_bucket(XRD, dec!(1000), Mock, &mut env)?;

    let account = create_account(&mut env, XRD)?;
    let account_proof = ProofFactory::create_fungible_proof(XRD, dec!(1), Mock, &mut env)?;
    LocalAuthZone::push(account_proof, &mut env)?;

    component.start_new_auction(&mut env)?;

    let current_auction: Auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");
    let time_buffer = get_state_auction_buffer(&mut env, component)?;

    env.set_current_time(
        current_auction
            .end_timestamp
            .add_minutes(1 - time_buffer as i64)
            .unwrap(),
    ); // Set time within time buffer
    let new_end_timestamp = env
        .get_current_time()
        .add_minutes(time_buffer as i64)
        .unwrap(); // This ends up being (previous endtime + 1 minute)

    // Act
    component.bid(xrd_bucket.into(), account, &mut env)?;

    // Assert
    let current_auction: Auction = component
        .get_current_auction(&mut env)
        .expect("Couldn't get active auction")
        .expect("No active auction");

    assert!(
        current_auction.end_timestamp == new_end_timestamp,
        "End timestamp was not changed"
    );

    Ok(())
}
