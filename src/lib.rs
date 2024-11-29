use scrypto::prelude::*;

#[blueprint]
mod radical_doge_rewards {

    enable_method_auth! {
        roles {
            airdropper => updatable_by: [OWNER];
        },
        methods {
            deposit_future_rewards => restrict_to: [OWNER];
            assign_deposited_rewards => restrict_to: [airdropper];
            assign_rewards => restrict_to: [airdropper];
        }
    }

    struct RadicalDogeRewards {
        future_rewards: FungibleVault,
        account_locker: Global<AccountLocker>,
    }

    impl RadicalDogeRewards {

        pub fn new(
            owner_badge_address: ResourceAddress,
            airdropper_badge_address: ResourceAddress,
            rewards_address: ResourceAddress,
        ) -> Global<RadicalDogeRewards> {

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(RadicalDogeRewards::blueprint_id());

            Self {
                future_rewards: FungibleVault::new(rewards_address),
                account_locker: Blueprint::<AccountLocker>::instantiate(
                    OwnerRole::Updatable(rule!(require(owner_badge_address))),
                    rule!(require(global_caller(component_address))),
                    rule!(require(owner_badge_address)),
                    rule!(deny_all),
                    rule!(require(owner_badge_address)),
                    None
                ),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                airdropper => rule!(require(airdropper_badge_address));
            ))
            .with_address(address_reservation)
            .globalize()
        }

        pub fn deposit_future_rewards(
            &mut self,
            future_rewards: FungibleBucket,
        ) {
            self.future_rewards.put(future_rewards);
        }

        pub fn assign_deposited_rewards(
            &mut self,
            rewards: IndexMap<Global<Account>, ResourceSpecifier>,
        ) {
            let mut total_coins = Decimal::ZERO;

            for resource_specifier in rewards.values() {
                match resource_specifier {
                    ResourceSpecifier::Fungible(amount) => total_coins += *amount,
                    ResourceSpecifier::NonFungible(_) => Runtime::panic("I can't handle NFTs".to_string()),
                }
            }

            let rewards_bucket = self.future_rewards.take_advanced(
                total_coins,
                WithdrawStrategy::Rounded(RoundingMode::AwayFromZero)
            );

            let remainings = self.account_locker.airdrop(
                rewards,
                rewards_bucket.into(),
                true
            );

            match remainings {
                Some(bucket) => self.future_rewards.put(FungibleBucket(bucket)),
                None => {},
            }
        }

        pub fn assign_rewards(
            &mut self,
            rewards: IndexMap<Global<Account>, ResourceSpecifier>,
            rewards_bucket: Bucket,
        ) -> Option<Bucket> {
            self.account_locker.airdrop(
                rewards,
                rewards_bucket,
                true
            )
        }
    }
}
