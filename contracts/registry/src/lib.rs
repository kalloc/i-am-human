use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::{env, near_bindgen, require, AccountId, PanicOnDefault};

use sbt::{TokenData, TokenId};

use crate::storage::*;

mod registry;
mod storage;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub admin: AccountId,

    /// registry of approved SBT contracts to issue tokens
    pub sbt_contracts: UnorderedMap<AccountId, CtrId>,
    pub ctr_id_map: LookupMap<CtrId, AccountId>, // reverse index
    /// registry of blacklisted accounts by issuer
    pub banlist: UnorderedSet<AccountId>,

    /// maps user account to list of token source info
    pub(crate) balances: LookupMap<AccountId, UnorderedMap<CtrClassId, TokenId>>,
    /// maps SBT contract -> map of tokens
    pub(crate) ctr_tokens: LookupMap<CtrTokenId, TokenData>,
    /// map of SBT contract -> next available token_id
    pub(crate) next_token_ids: LookupMap<CtrId, TokenId>,
    pub(crate) next_ctr_id: CtrId,
    pub(crate) ongoing_soul_tx: LookupMap<AccountId, CtrTokenId>,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        Self {
            admin,
            sbt_contracts: UnorderedMap::new(StorageKey::SbtContracts),
            ctr_id_map: LookupMap::new(StorageKey::SbtContractsRev),
            banlist: UnorderedSet::new(StorageKey::Banlist),
            balances: LookupMap::new(StorageKey::Balances),
            ctr_tokens: LookupMap::new(StorageKey::CtrTokens),
            next_token_ids: LookupMap::new(StorageKey::NextTokenId),
            next_ctr_id: 1,
            ongoing_soul_tx: LookupMap::new(StorageKey::OngoingSoultTx),
        }
    }

    //
    // Queries
    //

    pub fn sbt_contracts(&self) -> Vec<AccountId> {
        self.sbt_contracts.keys().collect()
    }

    //
    // Transactions
    //

    /// Transfers atomically all SBT tokens from one account to another account.
    /// The caller must be an SBT holder and the `to` must not be a banned account.
    /// Returns the lastly moved SBT identified by it's contract issuer and token ID as well
    /// a boolean: `true` if the whole process has finished, `false` when the process has not
    /// finished and should be continued by a subsequent call.
    /// User must keeps calling `sbt_soul_transfer` until `true` is returned.
    /// Must emit `SoulTransfer` event.
    #[payable]
    pub fn sbt_soul_transfer(&mut self, to: AccountId) -> (AccountId, TokenId, bool) {
        let start = self.ongoing_soul_tx.get(&to).unwrap_or(CtrTokenId {
            ctr_id: 0,
            token: 0,
        });
        println!("Starting at: {} {}", start.ctr_id, start.token);
        env::panic_str("not implemented");
        // TODO: lock `to` account if needed
    }

    //
    // Admin
    //

    /// returns false if the `issuer` contract was already registered.
    pub fn admin_add_sbt_issuer(&mut self, issuer: AccountId) -> bool {
        self.assert_admin();
        let previous = self.sbt_contracts.insert(&issuer, &self.next_ctr_id);
        self.ctr_id_map.insert(&self.next_ctr_id, &issuer);
        self.next_ctr_id += 1;
        previous.is_none()
    }

    //
    // Internal
    //

    pub(crate) fn ctr_id(&self, ctr: &AccountId) -> CtrId {
        // TODO: use Result rather than panic
        self.sbt_contracts.get(ctr).expect("SBT Issuer not found")
    }

    pub(crate) fn get_user_balances(&self, user: &AccountId) -> UnorderedMap<CtrClassId, TokenId> {
        self.balances
            .get(user)
            // TODO: verify how this works
            .unwrap_or_else(|| {
                UnorderedMap::new(StorageKey::BalancesMap {
                    owner: user.clone(),
                })
            })
    }

    /// updates the internal token counter based on how many tokens we want to mint (num), and
    /// returns the first valid TokenId for newly minted tokens.
    pub(crate) fn next_token_id(&mut self, ctr_id: CtrId, num: u64) -> TokenId {
        let tid = self.next_token_ids.get(&ctr_id).unwrap_or(0);
        self.next_token_ids.insert(&ctr_id, &(tid + num));
        tid + 1
    }

    #[inline]
    pub(crate) fn assert_not_banned(&self, owner: &AccountId) {
        require!(
            !self.banlist.contains(owner),
            format!("account {} is banned", owner)
        );
    }

    /// note: use ctr_id() if you need ctr_id
    pub(crate) fn assert_issuer(&self, contract: &AccountId) {
        require!(self.sbt_contracts.get(contract).is_some())
    }

    pub(crate) fn assert_admin(&self) {
        require!(self.admin == env::predecessor_account_id(), "not an admin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use sbt::{ClassId, SBTRegistry, TokenMetadata};

    use sbt::*;

    // TODO
    #[allow(dead_code)]

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice.near".to_string())
    }

    fn bob() -> AccountId {
        AccountId::new_unchecked("bob.near".to_string())
    }

    fn issuer1() -> AccountId {
        AccountId::new_unchecked("sbt1.near".to_string())
    }

    fn issuer2() -> AccountId {
        AccountId::new_unchecked("sbt2.near".to_string())
    }

    fn issuer3() -> AccountId {
        AccountId::new_unchecked("sbt3.near".to_string())
    }

    fn admin() -> AccountId {
        AccountId::new_unchecked("sbt.near".to_string())
    }

    fn mk_metadata(class: ClassId, expires_at: Option<u64>) -> TokenMetadata {
        TokenMetadata {
            class,
            issued_at: None,
            expires_at,
            reference: Some("abc".to_owned()),
            reference_hash: Some(vec![61, 61].into()),
        }
    }

    fn mk_token(token: TokenId, owner: AccountId, metadata: TokenMetadata) -> Token {
        Token {
            token,
            owner,
            metadata,
        }
    }

    fn mk_owned_token(token: TokenId, metadata: TokenMetadata) -> OwnedToken {
        OwnedToken { token, metadata }
    }

    const START: u64 = 10;

    fn setup(predecessor: &AccountId) -> (VMContext, Contract) {
        let mut ctx = VMContextBuilder::new()
            .predecessor_account_id(admin())
            // .attached_deposit(deposit_dec.into())
            .block_timestamp(START)
            .is_view(false)
            .build();
        testing_env!(ctx.clone());
        let mut ctr = Contract::new(admin());
        ctr.admin_add_sbt_issuer(issuer1());
        ctr.admin_add_sbt_issuer(issuer2());
        ctx.predecessor_account_id = predecessor.clone();
        testing_env!(ctx.clone());
        return (ctx, ctr);
    }

    #[test]
    fn mint() {
        let (mut ctx, mut ctr) = setup(&issuer1());
        let m1_1 = mk_metadata(1, Some(START + 10));
        let m1_2 = mk_metadata(1, Some(START + 12));
        let m2_1 = mk_metadata(2, Some(START + 14));
        let m4_1 = mk_metadata(4, Some(START + 16));

        let minted_ids = ctr.sbt_mint(vec![
            (alice(), vec![m1_1.clone()]),
            (bob(), vec![m1_2.clone()]),
            (alice(), vec![m2_1.clone()]),
        ]);
        assert_eq!(minted_ids, vec![1, 2, 3]);

        // mint again for Alice
        let minted_ids = ctr.sbt_mint(vec![(alice(), vec![m4_1.clone()])]);
        assert_eq!(minted_ids, vec![4]);

        // change the issuer and mint new tokens for alice
        ctx.predecessor_account_id = issuer2();
        testing_env!(ctx.clone());
        let minted_ids = ctr.sbt_mint(vec![(alice(), vec![m1_1.clone(), m2_1.clone()])]);
        // since we minted with different issuer, the new SBT should start with 1
        assert_eq!(minted_ids, vec![1, 2]);

        assert_eq!(4, ctr.sbt_supply(issuer1()));
        assert_eq!(2, ctr.sbt_supply(issuer2()));
        assert_eq!(0, ctr.sbt_supply(issuer3()));

        assert_eq!(3, ctr.sbt_supply_by_owner(alice(), issuer1(), None));
        assert_eq!(2, ctr.sbt_supply_by_owner(alice(), issuer2(), None));
        assert_eq!(1, ctr.sbt_supply_by_owner(bob(), issuer1(), None));
        assert_eq!(0, ctr.sbt_supply_by_owner(bob(), issuer2(), None));
        assert_eq!(0, ctr.sbt_supply_by_owner(issuer1(), issuer1(), None));

        let sbt1_1 = ctr.sbt(issuer1(), 1).unwrap();
        assert_eq!(sbt1_1, mk_token(1, alice(), m1_1.clone()));
        let sbt1_2 = ctr.sbt(issuer1(), 2).unwrap();
        assert_eq!(sbt1_2, mk_token(2, bob(), m1_2.clone()));
        let sbt1_3 = ctr.sbt(issuer1(), 3).unwrap();
        assert_eq!(sbt1_3, mk_token(3, alice(), m2_1.clone()));

        let sbt2_1 = ctr.sbt(issuer2(), 1).unwrap();
        assert_eq!(sbt2_1, mk_token(1, alice(), m1_1.clone()));

        let alice_sbts = ctr.sbt_tokens_by_owner(alice(), None, None, None);
        assert_eq!(
            alice_sbts,
            vec![
                (
                    issuer1(),
                    vec![
                        mk_owned_token(1, m1_1.clone()),
                        mk_owned_token(3, m2_1.clone()),
                        mk_owned_token(4, m4_1.clone())
                    ]
                ),
                (
                    issuer2(),
                    vec!(mk_owned_token(1, m1_1), mk_owned_token(2, m2_1))
                )
            ]
        );
    }
}
