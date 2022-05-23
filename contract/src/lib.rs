/* randomness-contract
 *
 * This contract demonstrates usage of random number generation in NEAR smart contracts.
 * 
 * The contract keeps track of a registry of counters and their owners. Counters have a
 * randomly generated UUID that represents them. Anyone who owns the counter can increment
 * or decrement its number by a random amount.
 */

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::ValidAccountId;
use near_sdk::{env, near_bindgen, PanicOnDefault};

use getrandom::register_custom_getrandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use uuid::Uuid;

fn fill_with_nothing(_dest: &mut [u8]) -> Result<(), getrandom::Error> {
    Ok(())
}

register_custom_getrandom!(fill_with_nothing);

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    seed: [u8; 32],
    counters: UnorderedMap<String, i32>,
    owners: UnorderedMap<String, ValidAccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            seed: env::sha256(&env::random_seed()).try_into().unwrap(),
            counters: UnorderedMap::new(b"c"),
            owners: UnorderedMap::new(b"o"),
        }
    }

    pub fn get_counter(&self, id: String) -> i32 {
        self._get_counter(&id)
    }

    pub fn get_owner(&self, id: String) -> ValidAccountId {
        self._get_owner(&id)
    }

    pub fn create_counter(&mut self) -> String {
        let caller = self._get_caller();
        self._add_entropy();
        let mut rng = ChaCha20Rng::from_seed(self.seed);

        let mut id_buf = [0u8; 16];
        rng.fill(&mut id_buf);
        let id = Uuid::from_slice(&id_buf)
            .unwrap()
            .simple()
            .to_string();

        let count = rng.gen();

        self.counters.insert(&id, &count);
        self.owners.insert(&id, &caller);

        id
    }

    pub fn inc_counter(&mut self, id: String) {
        self._check_owner(&id);
        self._add_entropy();
        let mut rng = ChaCha20Rng::from_seed(self.seed);

        let count = self._get_counter(&id);
        let inc = rng.gen_range(0i32..256);
        self.counters.insert(&id, &(count + inc));
    }

    pub fn dec_counter(&mut self, id: String) {
        self._check_owner(&id);
        self._add_entropy();
        let mut rng = ChaCha20Rng::from_seed(self.seed);

        let count = self._get_counter(&id);
        let dec = rng.gen_range(0i32..256);
        self.counters.insert(&id, &(count - dec));
    }

    fn _add_entropy(&mut self) {
        let mut data = Vec::new();
        data.extend_from_slice(&self.seed);
        data.extend_from_slice(&env::random_seed());
        data.extend_from_slice(&env::block_index().to_be_bytes());
        data.extend_from_slice(env::predecessor_account_id().as_bytes());
        self.seed = env::sha256(&data).try_into().unwrap();
    }

    fn _get_caller(&self) -> ValidAccountId {
        ValidAccountId::try_from(env::predecessor_account_id()).unwrap()
    }

    fn _get_counter(&self, id: &String) -> i32 {
        self.counters
            .get(id)
            .unwrap_or_else(|| env::panic(b"ERR_COUNTER_NOT_FOUND"))
    }

    fn _get_owner(&self, id: &String) -> ValidAccountId {
        self.owners.get(&id).unwrap_or_else(|| env::panic(b"ERR_COUNTER_NOT_FOUND"))
    }

    fn _check_owner(&self, id: &String) {
        let caller = ValidAccountId::try_from(env::predecessor_account_id()).unwrap();
        let owner = self._get_owner(id);
        if caller != owner {
            env::panic(b"ERR_CALLER_NOT_OWNER");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn predecessor() -> String {
        "alice.testnet".to_string()
    }

    fn get_context() -> VMContext {
        VMContext {
            current_account_id: "randomness.testnet".to_string(),
            signer_account_id: "bob.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: predecessor(),
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn create_counter() {
        let context = get_context();
        testing_env!(context);
        let mut contract = Contract::new();
        let id = contract.create_counter();
        assert_eq!(id, "67b75d2d1be8186127d3c3284d2ce27e", "Incorrect id.");
        let count = contract.get_counter(id.clone());
        assert_eq!(count, 1484363077, "Incorrect count.");
        let owner = contract.get_owner(id.clone()).to_string();
        assert_eq!(owner, predecessor(), "Incorrect owner.");
    }

    #[test]
    fn inc_counter() {
        let context = get_context();
        testing_env!(context);
        let mut contract = Contract::new();
        let id = contract.create_counter();
        let count = contract.get_counter(id.clone());
        contract.inc_counter(id.clone());
        let inc_count = contract.get_counter(id.clone());
        assert_eq!(inc_count - count, 173, "Incorrect increment.");
    }

    #[test]
    fn dec_counter() {
        let context = get_context();
        testing_env!(context);
        let mut contract = Contract::new();
        let id = contract.create_counter();
        let count = contract.get_counter(id.clone());
        contract.dec_counter(id.clone());
        let dec_count = contract.get_counter(id.clone());
        assert_eq!(count - dec_count, 173, "Incorrect decrement.");
    }
}
