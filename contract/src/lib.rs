use std::collections::HashMap;
use near_sdk::{
    near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, ext_contract, env, require, PromiseError, log, Balance, Gas, PromiseOrValue,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, UnorderedMap},
    serde::{Deserialize, Serialize},
};
use near_sdk::json_types::{U128};
use near_sdk::serde_json::{Map, Value};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::{NFT_METADATA_SPEC, NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata};


mod utils;
mod nft;
mod social;

use crate::nft::*;
use crate::social::*;
use crate::utils::*;

// const NEAR_SOCIAL_ACCOUNT_ID: &str = "v1.social08.testnet";
// const NEAR_SOCIAL_APP_NAME: &str = "social_avatars3";
// const NEAR_SOCIAL_APP_OWNER_ID: &str = "test_alice.testnet";
const NEAR_SOCIAL_ACCOUNT_ID: &str = "social.near";
const NEAR_SOCIAL_APP_NAME: &str = "avtr";
const NEAR_SOCIAL_APP_OWNER_ID: &str = "zavodil.near";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    Earnings,
    Applications
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    service_fee: FeeFraction,
    total_service_fee: Balance,
    total_spent: Balance,
    earnings: UnorderedMap<AccountId, Balance>,
    applications: UnorderedMap<AccountId, (Balance, Option<bool>)>,

    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    next_token_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Component {
    account_id: AccountId,
    category: String,
    value: String,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, service_fee: FeeFraction) -> Self {
        service_fee.assert_valid();
        Self {
            owner_id,
            service_fee,
            total_service_fee: 0,
            total_spent: 0,
            earnings: UnorderedMap::new( StorageKey::Earnings),
            applications: UnorderedMap::new(StorageKey::Applications),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                env::current_account_id(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&internal_get_metadata())),
            next_token_id: 0,
        }
    }

    pub fn execute(&mut self) {}

    #[payable]
    pub fn nft_mint(&mut self, receiver_id: AccountId, data: Vec<(AccountId, String, String)>) -> PromiseOrValue<Option<TokenId>> {
        let account_id = env::predecessor_account_id();
        require!(receiver_id == account_id, "Illegal receiver");

        let colors_names = vec!["background".to_string(), "clothingColor".to_string(), "hairColor".to_string(), "skin".to_string(), "clothingGraphicsColor".to_string(), "facialHairColor".to_string(), "hatColor".to_string(), "accessoriesColor".to_string(), "svgBackground".to_string()];

        let keys: Vec<String>  = data.iter().fold(vec![format!("{}/{}/whitelist/**", NEAR_SOCIAL_APP_OWNER_ID, NEAR_SOCIAL_APP_NAME)], |mut result, (account_id, category, value)| {
            let request_key =
            if colors_names.contains(category) {
                format!("{}/{}/colors/{}/*", account_id, NEAR_SOCIAL_APP_NAME, value)
            } else {
                format!("{}/{}/components/{}/{}/*", account_id, NEAR_SOCIAL_APP_NAME, category, value)
            };

            if !result.contains(&request_key) {
                result.push(request_key);
            }

            result
        });

        ext_social::ext(AccountId::new_unchecked(NEAR_SOCIAL_ACCOUNT_ID.to_string()))
            .with_static_gas(GAS_FOR_SOCIAL_GET)
            .get(
                keys,
                None,
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_AFTER_SOCIAL_GET)
                    .with_attached_deposit(env::attached_deposit())
                    .after_social_get(receiver_id, data)
            ).into()
    }


    #[payable]
    #[private]
    pub fn after_social_get(
        &mut self,
        #[callback_result] value: Result<Value, PromiseError>,
        receiver_id: AccountId,
        options: Vec<(AccountId, String, String)>,
    )  -> Option<TokenId> {
        if let Ok(mut value) = value {
            let keys = value.as_object_mut().expect("Data is not a JSON object");

            let mut requests = HashMap::<String, String>::new();
            for option in options {
                requests.insert(option.1, option.2);
            }

            let mut data = HashMap::<&String, String>::new();
            let mut total_cost: Balance = 0;
            let mut payments: HashMap<AccountId, Balance> = HashMap::new();
            let mut whitelist = vec![AccountId::new_unchecked(NEAR_SOCIAL_APP_OWNER_ID.to_string())];

            for (account, value) in keys.clone() {
                if account.as_str() == NEAR_SOCIAL_APP_OWNER_ID {
                    for (kind, kind_data) in value.get(NEAR_SOCIAL_APP_NAME.to_string()).expect("Missing data").as_object().expect("Missing avatar kind") {
                        if kind == "whitelist" {
                            for (item, _) in kind_data.as_object().unwrap() {
                                whitelist.push(AccountId::new_unchecked(item.to_string()));
                            }
                        }
                    }
                }
            }

            for (account, value) in keys {
                let account_id = AccountId::new_unchecked(account.to_owned());

                // colors, components, whitelist
                for (kind, kind_data) in value.get(NEAR_SOCIAL_APP_NAME.to_string()).expect("Missing data").as_object().expect("Missing avatar kind") {
                    if kind != "whitelist" {
                        if whitelist.contains(&account_id) {
                            // pale, clothing
                            for (category, category_data) in kind_data.as_object().unwrap() {
                                let is_color: bool = kind == "colors";

                                // [color item], graphicShirt
                                for (category_value, value_data) in category_data.as_object().unwrap() {
                                    if is_color {
                                        if category_value == "src" {
                                            data.insert(category, value_data.as_str().unwrap().to_string());
                                        }
                                    } else {
                                        for (item, item_data) in value_data.as_object().unwrap() {
                                            if item == "src" {
                                                data.insert(category, item_data.as_str().unwrap().to_string());
                                            } else if item == "price" {
                                                let price = item_data.as_str().unwrap().parse::<u128>().unwrap();
                                                if price > 0 {
                                                    total_cost += price;
                                                    let prev_payment_balance: Balance = payments.get(&account_id).unwrap_or(&0u128).to_owned();
                                                    payments.insert(account_id.clone(), prev_payment_balance + price);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        else {
                            log!("Skip data from {}", account_id);
                        }
                    }
                }
            }


            require!(total_cost <= env::attached_deposit() + DEPOSIT_FOR_SOCIAL_SET, "ERR_DEPOSIT_IS_NOT_ENOUGH");
            self.total_spent += total_cost;

            let skin_color = data.get(requests.get(&"skin".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");
            let hat_color = data.get(requests.get(&"hatColor".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");
            let hair_color = data.get(requests.get(&"hairColor".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");
            let facial_hair_color = data.get(requests.get(&"facialHairColor".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");
            let clothe_color = data.get(requests.get(&"clothingColor".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");
            let clothing_graphics_color = data.get(requests.get(&"clothingGraphicsColor".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");
            let accessories_color = data.get(requests.get(&"accessoriesColor".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA");

            let nose_type = group(
                &r##"<path fill-rule="evenodd" clip-rule="evenodd" d="M16 8c0 4.418 5.373 8 12 8s12-3.582 12-8" fill="#000" fill-opacity=".16"/>"##.to_string(),
                112, 122, false);
            let skin_type = group(
                &r##"<path d="M100 0C69.072 0 44 25.072 44 56v6.166c-5.675.952-10 5.888-10 11.834v14c0 6.052 4.48 11.058 10.305 11.881 2.067 19.806 14.458 36.541 31.695 44.73V163h-4c-39.764 0-72 32.236-72 72v9h200v-9c0-39.764-32.236-72-72-72h-4v-18.389c17.237-8.189 29.628-24.924 31.695-44.73C161.52 99.058 166 94.052 166 88V74c0-5.946-4.325-10.882-10-11.834V56c0-30.928-25.072-56-56-56z" fill="%SKIN_COLOR%"/><path d="M76 144.611v8A55.79 55.79 0 00100 158a55.789 55.789 0 0024-5.389v-8A55.789 55.789 0 01100 150a55.79 55.79 0 01-24-5.389z" fill="#000" fill-opacity=".1"/>"##
                    .replace("%SKIN_COLOR%", skin_color),
                40, 36, false);

            let top_type_z_index = get_top_type(requests.get(&"top".to_string()).expect("ERR_NO_DATA"));

            let top_type = group(
                &data.get(&"top".to_string()).expect("ERR_NO_DATA")
                    .replace("%TOP_COLOR_1%", hat_color)
                    .replace("%TOP_COLOR_2%", hair_color),
                7, 0, false);

            let clothing_graphic = data.get(&"clothingGraphic".to_string()).expect("ERR_NO_DATA");

            let facial_hair_type = group(
                &data.get(&"facialHair".to_string()).expect("ERR_NO_DATA")
                    .replace("%FACIALHAIR_COLOR_1%", facial_hair_color),
                56, 72, true);

            let clothe_type = group(
                &data.get(&"clothing".to_string()).expect("ERR_NO_DATA")
                    .replace("%CLOTHING_COLOR_1%", clothe_color)
                    .replace("%CLOTHING_GRAPHICS%", clothing_graphic)
                    .replace("%CLOTHING_GRAPHICS_COLOR%", clothing_graphics_color),
                8, 170, false);

            let eye_type = group(data.get(&"eyes".to_string()).expect("ERR_NO_DATA"),
                                 84, 90, false);
            let eyebrow_type = group(data.get(&"eyebrows".to_string()).expect("ERR_NO_DATA"),
                                     84, 82, false);

            let mouth_type = group(
                data.get(&"mouth".to_string()).expect("ERR_NO_DATA"),
                86, 134, false);

            let accessories_type = group(
                &data.get(&"accessories".to_string()).expect("ERR_NO_DATA")
                    .replace("%ACCESSORIES_COLOR_1%", accessories_color),
                69, 85, true);

            let top_0 = if top_type_z_index == 0 { top_type.clone() } else { "".to_string() };
            let top_1 = if top_type_z_index == 1 { top_type.clone() } else { "".to_string() };
            let top_2 = if top_type_z_index == 2 { top_type } else { "".to_string() };

            let content = format!("{}{}{}{}{}{}{}{}{}{}{}",
                                  skin_type, clothe_type, mouth_type, nose_type, eye_type,
                                  eyebrow_type, top_0, facial_hair_type, top_1, accessories_type, top_2);

            let mask_id = get_binary_mask_id();
            let mask_id_url = format!("url(#{})", mask_id);

            let svg_background = format!(r#"<path fill="{}" d="M0 0h280v280H0z"/>"#, data.get(requests.get(&"svgBackground".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA"));
            let background = format!(r#"<path d="M260 160c0 66.274-53.726 120-120 120S20 226.274 20 160 73.726 40 140 40s120 53.726 120 120z" fill="{}"/>"#, data.get(requests.get(&"background".to_string()).expect("ERR_NO_DATA")).expect("ERR_NO_DATA"));
            let mask = format!(r##"<mask id="{}" maskUnits="userSpaceOnUse" x="8" y="0" width="264" height="280"><path fill-rule="evenodd" clip-rule="evenodd" d="M272 0H8v160h12c0 66.274 53.726 120 120 120s120-53.726 120-120h12V0z" fill="#fff"/></mask><g mask="{}">{}</g>"##, mask_id, mask_id_url, content);
            let image = format!("{}{}{}", svg_background, background, mask);
            let svg = format!(r##"<svg viewBox="0 0 280 280" fill="none" xmlns="http://www.w3.org/2000/svg">{}</svg>"##, image);

            // log!("svg {:?}", svg);

            log!("token_id: {}", self.next_token_id);

            let new_token = self.internal_mint(&receiver_id, &svg, total_cost);

            let mut total_service_fee: Balance = 0;
            for (payment_receiver_id, payment) in payments.clone() {
                let service_fee: Balance = self.service_fee.multiply(payment);
                total_service_fee += service_fee;
                let artist_earnings = payment - service_fee;
                let prev_artist_earnings = self.earnings.get(&payment_receiver_id).unwrap_or_default();
                self.earnings.insert(&payment_receiver_id, &(prev_artist_earnings + artist_earnings));
                Promise::new(payment_receiver_id).transfer(artist_earnings);
            }

            if total_service_fee > 0 {
                self.total_service_fee += total_service_fee;
            }

            // todo collect all notifications and push
            self.internal_social_index_nofity(&new_token.token_id, payments);

            Some(new_token.token_id)
        }
        else {
            None
        }
    }
}

fn group(content: &String, x: u16, y: u16, avoid_empty: bool) -> String {
    if avoid_empty && content.is_empty() {
        return "".to_string();
    }
    format!("<g transform=\"translate({}, {})\">{}</g>", x, y, content)
}

fn get_top_type(option: &str) -> u32 {
    match option {
        "bigHair" | "bob" | "curly" | "curvy" | "dreads" | "frida" | "fro" | "froAndBand" | "miaWallace" | "longButNotTooLong" |     "shavedSides" | "straight01" | "straight02" |"straightAndStrand" |"dreads01" | "dreads02" | "shaggyMullet"=> 0,
        "bun" | "frizzle" | "shortCurly" | "shortFlat" | "shortRound" | "shortWaved" | "sides" | "theCaesar" | "theCaesarAndSidePart" | "hat" | "hijab" | "turban" | "eyepatch" => 1,
        "shaggy" | "winterHat01" | "winterHat02" | "winterHat03" |"winterHat04" => 2,
        &_ => 1
    }
}

pub fn get_binary_mask_id() -> String {
    let random_seed = env::random_seed();
    format!("{}{}{}{}", random_seed[0], random_seed[1], random_seed[2], random_seed[3])
}
