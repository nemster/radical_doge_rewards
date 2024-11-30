use scrypto::prelude::*;

/* This blueprint allows the owner to lock a fungible coin and to distribute it later as a reward.
 *
 * Only the owner can deposit the rewards, he can do it multiple times.
 *
 * The holder of the airdropper badge can input an IndexMap (Account => Amount) to distribute part
 * of the rewards previously deposited by the owner.
 *
 * The distribution happens through an AccountLocker, the airdropper is also allowed to interact
 * directly with the AccountLocker to distribute any different reward than the one deposited
 * in this component.
*/

#[blueprint]
mod radical_doge_rewards {

    enable_method_auth! {
        roles {
            airdropper => updatable_by: [OWNER];
        },
        methods {
            deposit_future_rewards => restrict_to: [OWNER];
            distribute_deposited_rewards => restrict_to: [airdropper];
        }
    }

    struct RadicalDogeRewards {

        // The vault to contain the future rewards
        future_rewards: FungibleVault,

        // The AccountLocker to distribute the rewards
        account_locker: Global<AccountLocker>,
    }

    impl RadicalDogeRewards {

        // Instantiate a globalized RadicalDogeRewards component
        pub fn new(
            owner_badge_address: ResourceAddress,
            airdropper_badge_address: ResourceAddress,
            rewards_address: ResourceAddress,
        ) -> Global<RadicalDogeRewards> {

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(RadicalDogeRewards::blueprint_id());

            // Both this component and the holder of the airdropper_badge are authorized to deposit in the AccountLocker
            let storer_role = rule!(require(CompositeRequirement::AnyOf(vec![
                    global_caller(component_address).into(),
                    require(airdropper_badge_address),
            ])));

            // Instantiate an AccountLocker to distribute rewards
            let account_locker = Blueprint::<AccountLocker>::instantiate(
                OwnerRole::Updatable(rule!(require(owner_badge_address))), // owner_role
                storer_role,
                rule!(require(owner_badge_address)), // storer_updater_role
                rule!(deny_all), // recoverer_role
                rule!(require(owner_badge_address)), // recoverer_updater_role
                None
            );

            Self {
                future_rewards: FungibleVault::new(rewards_address),
                account_locker: account_locker,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                airdropper => rule!(require(airdropper_badge_address));
            ))
            .with_address(address_reservation)
            .globalize()
        }

        // Lock future rewards in the component
        pub fn deposit_future_rewards(
            &mut self,
            future_rewards: FungibleBucket,
        ) {
            self.future_rewards.put(future_rewards);
        }

        // Distribute previously deposited rewards using an AccountLocker
        pub fn distribute_deposited_rewards(
            &mut self,

            // Account => ResourceSpecifier::Fungible(Decimal)
            rewards: IndexMap<Global<Account>, ResourceSpecifier>,
        ) {

            // Compute the total amount to distribute
            let mut total_coins = Decimal::ZERO;
            for resource_specifier in rewards.values() {
                match resource_specifier {
                    ResourceSpecifier::Fungible(amount) => total_coins += *amount,
                    ResourceSpecifier::NonFungible(_) => Runtime::panic("I can't handle NFTs".to_string()),
                }
            }

            // Take all of the rewards to distribute out of the vault
            let rewards_bucket = self.future_rewards.take_advanced(
                total_coins,
                WithdrawStrategy::Rounded(RoundingMode::AwayFromZero)
            );

            // Distribute rewards and get back any remaining coin due to divisibility rounding
            let remainings = self.account_locker.airdrop(
                rewards,
                rewards_bucket.into(),
                true
            );

            // If there are remaining coins put them back into the vault
            match remainings {
                Some(bucket) => self.future_rewards.put(FungibleBucket(bucket)),
                None => {},
            }
        }
    }
}
