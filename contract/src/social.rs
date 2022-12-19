use crate::*;

pub const GAS_FOR_SOCIAL_GET: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_SOCIAL_SET: Gas = Gas(Gas::ONE_TERA.0 * 40);
pub const GAS_FOR_AFTER_SOCIAL_GET: Gas = Gas(Gas::ONE_TERA.0 * 80);
pub const DEPOSIT_FOR_SOCIAL_SET: Balance = 50_000_000_000_000_000_000_000;

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct GetOptions {
    pub with_block_height: Option<bool>,
    pub with_node_id: Option<bool>,
    pub return_deleted: Option<bool>,
}

#[ext_contract(ext_social)]
pub trait ExtSocial {
    fn get(self, keys: Vec<String>, options: Option<GetOptions>) -> Value;
    fn set(&mut self, data: Value);
}

#[ext_contract(ext_self)]
pub trait ExtContract {
    fn after_social_get(&mut self, #[callback_result] value: Result<Value, PromiseError>, receiver_id: AccountId, options: Vec<(AccountId, String, String)>) -> Option<TokenId>;
}

impl Contract {
    pub fn internal_social_index_nofity(&mut self, token_id: &TokenId, payments: HashMap<AccountId, Balance>) {
        // "notify": "[{\"key\":\"root.near\",\"value\":{\"type\":\"follow\"}},{\"key\":\"mob.near\",\"value\":{\"type\":\"follow\"}}]"

        let index: String =  payments
            .iter()
            .map(|(account_id, amount)| format!(r#"{{"key":"{}","value":{{"type":"purchase","amount":"{}","token_id":"{}"}}}}"#,
                                        account_id, amount, token_id))
            .collect::<Vec<String>>()
            .join(",");

        let mut notify_data: Map<String, Value> = Map::new();
        notify_data.insert("notify".to_string(), Value::String(format!("[{}]", index)));

        let mut index_data: Map<String, Value> = Map::new();
        index_data.insert("index".to_string(), Value::Object(notify_data));

        let mut data: Map<String, Value> = Map::new();
        data.insert(env::current_account_id().to_string(), Value::Object(index_data));

        ext_social::ext(AccountId::new_unchecked(NEAR_SOCIAL_ACCOUNT_ID.to_string()))
            .with_static_gas(GAS_FOR_SOCIAL_SET)
            .with_attached_deposit(DEPOSIT_FOR_SOCIAL_SET)
            .set(
                Value::Object(data)
            );
    }

    /*
    pub fn internal_social_set_token_id(&mut self, token_id: &TokenId, account_id: &AccountId) {
        // NEAR_SOCIAL_APP_NAME >> tokens >> holder.near >> [token_id1: "", token_id2: ""]
        let mut token_data: Map<String, Value> = Map::new();
        token_data.insert(token_id.to_string(), Value::String("".to_string()));

        let mut account_data: Map<String, Value> = Map::new();
        account_data.insert(account_id.to_string(), Value::Object(token_data));

        let mut tokens_data: Map<String, Value> = Map::new();
        tokens_data.insert("tokens".to_string(), Value::Object(account_data));

        let mut app_data: Map<String, Value> = Map::new();
        app_data.insert(NEAR_SOCIAL_APP_NAME.to_string(), Value::Object(tokens_data));

        let mut data: Map<String, Value> = Map::new();
        data.insert(env::current_account_id().to_string(), Value::Object(app_data));

        ext_social::ext(AccountId::new_unchecked(NEAR_SOCIAL_ACCOUNT_ID.to_string()))
            .with_static_gas(GAS_FOR_SOCIAL_SET)
            .with_attached_deposit(DEPOSIT_FOR_SOCIAL_SET)
            .set(
                Value::Object(data)
            );
    }*/
}