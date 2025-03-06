use mock_oci_pool::mockocipool_test::*;
use piers_rugyard::piers_rugyard_test::*;
use piers_rugyard::types::*;
use scrypto::prelude::Url;
use scrypto_test::prelude::*;

/// Creates an account reference with an owner role we can actually create a proof of
pub fn create_account(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    owner_resource: ResourceAddress,
) -> Result<Reference, RuntimeError> {
    let owner_role = OwnerRole::Fixed(rule!(require(owner_resource)));
    let account = env
        .call_function_typed::<_, AccountCreateAdvancedOutput>(
            ACCOUNT_PACKAGE,
            ACCOUNT_BLUEPRINT,
            ACCOUNT_CREATE_ADVANCED_IDENT,
            &AccountCreateAdvancedInput {
                owner_role,
                address_reservation: None,
            },
        )?
        .0
        .into();

    Ok(account)
}

/// Creates a basic test environment
pub fn create_test_environment() -> Result<
    (
        TestEnvironment<InMemorySubstateDatabase>,
        PiersRugyard,
        Bucket,
    ),
    RuntimeError,
> {
    let mut env = TestEnvironment::new();

    // Create mock stuff
    let owner_resource_bucket =
        ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(1, &mut env)?;
    let owner_resource_address = owner_resource_bucket.0.resource_address(&mut env)?;

    let (_oci_pool, oci_pool_address, early_resource_address) = instantiate_oci_pool(&mut env)?;

    // Instantiate PiersRugyard component
    let nft_collection_package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;
    let component = PiersRugyard::instantiate(
        360,
        5,
        dec!(50),
        owner_resource_address,
        oci_pool_address,
        early_resource_address,
        nft_collection_package_address,
        &mut env,
    )?;

    Ok((env, component, owner_resource_bucket.into()))
}

pub fn create_prepared_test_environment() -> Result<
    (
        TestEnvironment<InMemorySubstateDatabase>,
        PiersRugyard,
        Bucket,
    ),
    RuntimeError,
> {
    let (mut env, mut component, owner_resource) = create_test_environment()?;

    // Push owner proof to auth zone
    let proof = owner_resource.create_proof_of_all(&mut env)?;
    LocalAuthZone::push(proof, &mut env)?;

    // Mint NFTs
    component.mint_nft(
        NFT {
            key_image_url: Url::of("https://www.google.com/"),
            name: "My NFT!".to_string(),
            attribute1: "Worm".to_string(),
            attribute2: "Gears changed".to_string(),
        },
        &mut env,
    )?;

    component.mint_nft(
        NFT {
            key_image_url: Url::of("https://www.google.com/2"),
            name: "My NFT 2!".to_string(),
            attribute1: "Redhead".to_string(),
            attribute2: "Mojo".to_string(),
        },
        &mut env,
    )?;

    // Activate auctions
    component.flip_status(&mut env)?;

    // Clear the auth zone
    LocalAuthZone::drop_regular_proofs(&mut env)?;

    Ok((env, component, owner_resource))
}

// Instantiates a mock OCI pool
pub fn instantiate_oci_pool(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
) -> Result<(MockOciPool, ComponentAddress, ResourceAddress), RuntimeError> {
    let early_bucket =
        ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100_000_000, env)?;
    let early_resource_address = early_bucket.0.resource_address(env)?;

    let xrd_bucket =
        BucketFactory::create_fungible_bucket(XRD, dec!(100_000_000), CreationStrategy::Mock, env)?;

    // Instantiate mock Ociswap pool
    let mock_oci_pool_package_address =
        PackageFactory::compile_and_publish("mock_oci_pool", env, CompileProfile::Fast)?;
    let (oci_pool, oci_pool_address) = MockOciPool::instantiate(
        xrd_bucket.into(),
        early_bucket.into(),
        mock_oci_pool_package_address,
        env,
    )?;

    Ok((oci_pool, oci_pool_address, early_resource_address))
}

/// Helper function to mint an NFT
pub fn mint_nft(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    mut component: PiersRugyard,
) -> Result<(), RuntimeError> {
    component.mint_nft(
        NFT {
            key_image_url: Url::of("https://www.google.com/"),
            name: "My NFT!".to_string(),
            attribute1: "Worm".to_string(),
            attribute2: "Gears changed".to_string(),
        },
        env,
    )?;

    Ok(())
}

///---- State helpers -----///
pub fn get_state_available_nfts_list(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<Vec<NonFungibleLocalId>, RuntimeError> {
    let available_nfts_list = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.available_nfts_list.clone()
        })?;

    Ok(available_nfts_list)
}

pub fn get_state_resource_address(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<ResourceAddress, RuntimeError> {
    let resource_address = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.nft_manager.address()
        })?;

    Ok(resource_address)
}

pub fn get_state_active(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<bool, RuntimeError> {
    let active = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.active
        })?;

    Ok(active)
}

pub fn get_state_pool_address(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<ComponentAddress, RuntimeError> {
    let early_pool = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.early_pool
        })?;

    Ok(early_pool)
}

pub fn get_state_auction_duration(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<u64, RuntimeError> {
    let auction_duration = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.auction_duration_minutes
        })?;

    Ok(auction_duration)
}

pub fn get_state_auction_buffer(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<u64, RuntimeError> {
    let auction_buffer = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.auction_buffer_minutes
        })?;

    Ok(auction_buffer)
}

pub fn get_state_minimum_bid_increase(
    env: &mut TestEnvironment<InMemorySubstateDatabase>,
    component: PiersRugyard,
) -> Result<Decimal, RuntimeError> {
    let minimum_bid_increase = env
        .with_component_state::<PiersRugyardState, _, _, _>(component, |state, _env| {
            state.minimum_bid_increase
        })?;

    Ok(minimum_bid_increase)
}
