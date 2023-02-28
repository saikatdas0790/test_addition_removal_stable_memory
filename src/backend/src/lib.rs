use std::{cell::RefCell, collections::HashSet};

use candid::{export_service, CandidType, Principal};
use ic_cdk::export::serde::Deserialize;
use ic_stable_memory::utils::ic_types::SPrincipal;
use serde::{Deserializer, Serialize};

mod test;
mod upgrader;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[derive(Default, Serialize, Deserialize, CandidType)]
pub struct CanisterData {
    inner_struct: InnerStruct,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct InnerStruct {
    #[serde(deserialize_with = "principal_deserializer")]
    some_set: HashSet<Principal>,
}

fn principal_deserializer<'de, D>(deserializer: D) -> Result<HashSet<Principal>, D::Error>
where
    D: Deserializer<'de>,
{
    let previous: HashSet<SPrincipal> = HashSet::deserialize(deserializer)?;

    Ok(previous.into_iter().map(|principal| principal.0).collect())
}

impl Default for InnerStruct {
    fn default() -> Self {
        Self {
            some_set: HashSet::new(),
        }
    }
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn add_new_principal(principal: Principal) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        canister_data.inner_struct.some_set.insert(principal);
    });
}

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_principal() -> Vec<Principal> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();
        canister_data
            .inner_struct
            .some_set
            .iter()
            .cloned()
            .collect()
    })
}

const BUFFER_SIZE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();
        upgrader::serialize_to_stable_memory(canister_data, BUFFER_SIZE_BYTES)
            .expect("Failed to serialize canister data");
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    match upgrader::deserialize_from_stable_memory::<CanisterData>(BUFFER_SIZE_BYTES) {
        Ok(canister_data) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(e) => {
            ic_cdk::print(format!("Error: {:?}", e));
            panic!("Failed to restore canister data from stable memory");
        }
    }
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
