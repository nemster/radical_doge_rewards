# Radical Doge rewards

This blueprint allows the owner to lock a fungible coin and to distribute it later as a reward. Only the owner can deposit the rewards, he can do it multiple times.  
The holder of the airdropper badge can input an IndexMap (Account => Amount) to distribute part of the rewards previously deposited by the owner.  
The distribution happens through an AccountLocker, the airdropper is also allowed to interact directly with the AccountLocker to distribute any different reward than the one deposited in this component.  

## Known bugs and limitations

It is not possible to distribute rewards to more than about 100 recipients in a single transaction due to Radix network transaction limits.  

## Transaction manifests

### new

Instantiate a globalized RadicalDogeRewards component.  
Looking at the transaction details you will see that a `locker_...` component has been created too.  

```
CALL_FUNCTION
    Address("")
    "RadicalDogeRewards"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<AIRDROPPER_BADGE_ADDRESS>")
    Address("<REWARDS_ADDRESS>")
;
```

`<OWNER_BADGE_ADDRESS>` Resource address of the owner badge.  
`<AIRDROPPER_BADGE_ADDRESS>` Resource address of the airdropper badge.  
`<REWARDS_ADDRESS>` Resource address of the coin that will be distributed as a reward.  

### deposit_future_rewards

Lock future rewards in the component. Only the owner is allowed call this method.  

```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<REWARDS_ADDRESS>")
    Decimal("<REWARDS_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<REWARDS_ADDRESS>")
    Bucket("future_rewards")
;
CALL_METHOD
    Address("<DOGE_REWARDS_COMPONENT_ADDRESS>")
    "deposit_future_rewards"
    Bucket("future_rewards")
;
```

`<ACCOUNT>` The account holding the owner badge and the rewards to deposit.  
`<OWNER_BADGE_ADDRESS>` Resource address of the owner badge (I assumed it is a fungible resource).  
`<REWARDS_ADDRESS>` Resource address of the coin that will be distributed as a reward.  
`<REWARDS_AMOUNT>` The amount of rewards to deposit.  
`<DOGE_REWARDS_COMPONENT_ADDRESS>` The address of the RadicalDogeRewards component.  

### distribute_deposited_rewards

Distribute previously deposited rewards using an AccountLocker. Only the airdropper is allowed to call this method.  
In order to build the list of recipents I suggest to use [this API](https://radix-babylon-gateway-api.redoc.ly/#operation/ResourceHoldersPage) to get the holders of the LP token. In case of DOGE the OciSwap LP token is `resource_rdx1t47kxqvrzfj8vl4f2tpm0d9fy8gjt74nhxn52g33tu6urlgh0y3kuf`.  
It is also possible to schedule this operation without human interaction by creating a bot that uses a [typescript radix wallet](https://github.com/radixdlt/experimental-examples/tree/main/typescript-wallet) to execute the transactions. The account just needs the airdropper badge and some XRD to pay the fees.  

```
CALL_METHOD
	Address("<ACCOUNT>")
	"create_proof_of_amount"
	Address("<AIRDROPPER_BADGE_ADDRESS>")
	Decimal("1")
;
CALL_METHOD
	Address("<DOGE_REWARDS_COMPONENT_ADDRESS>`")
	"distribute_deposited_rewards"
	Map<Address, Enum>(
        Address("<RECIPIENT>") => Enum<0u8>(Decimal("<AMOUNT>")),
        ...
    )
;
```

`<ACCOUNT>` The account holding the airdroppper badge.  
`<AIRDROPPER_BADGE_ADDRESS>` Resource address of the airdropper badge (I assumed it is a fungible resource).  
`<DOGE_REWARDS_COMPONENT_ADDRESS>` The address of the RadicalDogeRewards component.  
`<RECIPIENT>` The account address of one of the recipents of the airdrop.  
`<AMOUNT>` The amount of rewards for the recipent.  

