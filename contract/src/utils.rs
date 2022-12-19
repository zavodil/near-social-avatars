use crate::*;

type WrappedBalance = U128;

pub const APPLICATION_DEPOSIT: Balance = 10_000_000_000_000_000_000_000_000; // 10 NEAR

#[near_bindgen]
impl Contract {
    pub fn get_earning(&self, account_id: AccountId) -> WrappedBalance {
        self.earnings.get(&account_id).expect("ERR_NO_DATA").into()
    }

    pub fn get_earnings(&self,  from_index: Option<u64>, limit: Option<u64>) -> Vec<(AccountId, WrappedBalance)> {
        unordered_map_pagination(&self.earnings, from_index, limit)
    }

    pub fn get_application(&self, account_id: AccountId) -> (WrappedBalance, Option<bool>) {
        let application = self.applications.get(&account_id).expect("ERR_NO_DATA");
        (WrappedBalance::from(application.0), application.1)
    }

    pub fn get_applications(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(AccountId, (Balance, Option<bool>))>{
        unordered_map_pagination(&self.applications, from_index, limit)
    }

    #[payable]
    pub fn add_application(&mut self) {
        require!(env::attached_deposit() >= APPLICATION_DEPOSIT, "ERR_DEPOSIT_IS_NOT_ENOUGH");
        self.applications.insert(&env::predecessor_account_id(), &(env::attached_deposit(), None));
    }
}

pub(crate) fn unordered_map_pagination<K, VV, V>(
    m: &UnorderedMap<K, VV>,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<(K, V)>
    where
        K: BorshSerialize + BorshDeserialize,
        VV: BorshSerialize + BorshDeserialize,
        V: From<VV>,
{
    let keys = m.keys_as_vector();
    let values = m.values_as_vector();
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(keys.len());
    (from_index..std::cmp::min(keys.len(), from_index + limit))
        .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap().into()))
        .collect()
}

use uint::construct_uint;
construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl FeeFraction {
    pub fn assert_valid(&self) {
        assert_ne!(self.denominator, 0, "Denominator must be a positive number");
        assert!(
            self.numerator <= self.denominator,
            "The treasure fee must be less or equal to 1"
        );
    }

    pub fn multiply(&self, value: Balance) -> Balance {
        (U256::from(self.numerator) * U256::from(value) / U256::from(self.denominator)).as_u128()
    }
}
