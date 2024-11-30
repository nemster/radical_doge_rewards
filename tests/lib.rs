use scrypto_test::prelude::*;
use scrypto::blueprints::locker::ResourceSpecifier;
use radical_doge_rewards::radical_doge_rewards_test::*;

#[test]
fn test_radical_doge_rewards_with_test_environment() -> Result<(), RuntimeError> {

    let mut env = TestEnvironment::new();
    env.disable_auth_module();

    // Create the owner badge
    let owner_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)?;
    let owner_badge_address = owner_badge_bucket.resource_address(&mut env)?;

    // Create the airdroppper badge
    let airdropper_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)?;
    let airdropper_badge_address = airdropper_badge_bucket.resource_address(&mut env)?;

    // Create DOGE coin
    let doge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(143000000000i64, &mut env)?;
    let doge_address = doge_bucket.resource_address(&mut env)?;

    // Instantiate the RadicalDogeRewards component
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;
    let mut radical_doge_rewards = RadicalDogeRewards::new(
        owner_badge_address,
        airdropper_badge_address,
        doge_address,
        package_address,
        &mut env
    )?;

    // Deposit 10000 DOGE in the component
    radical_doge_rewards.deposit_future_rewards(
        doge_bucket.take(dec![10000], &mut env)?,
        &mut env
    )?;

    // Create an account to receive the rewards
    let account = env.call_function_typed::<_, AccountCreateOutput>(
        ACCOUNT_PACKAGE,
        ACCOUNT_BLUEPRINT,
        ACCOUNT_CREATE_IDENT,
        &AccountCreateInput {},
    )?.0;
    let account_address = account.0;

    // Create another account to receive the rewards
    let another_account = env.call_function_typed::<_, AccountCreateOutput>(
        ACCOUNT_PACKAGE,
        ACCOUNT_BLUEPRINT,
        ACCOUNT_CREATE_IDENT,
        &AccountCreateInput {},
    )?.0;
    let another_account_address = another_account.0;

    // Assign rewards to accounts
    let mut recipients: IndexMap<Reference, ResourceSpecifier> = IndexMap::new();
    let rewards_to_account = dec![100];
    recipients.insert(account_address.into(), ResourceSpecifier::Fungible(rewards_to_account));
    let rewards_to_another_account = dec![200];
    recipients.insert(another_account_address.into(), ResourceSpecifier::Fungible(rewards_to_another_account));

    // Distribute the rewards to the accounts
    radical_doge_rewards.distribute_deposited_rewards(recipients, &mut env)?;

    // TODO: how to verify the received amounts?

    Ok(())
}
