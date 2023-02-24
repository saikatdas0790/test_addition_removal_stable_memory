use std::cell::RefCell;

use candid::{export_service, CandidType, Principal};
use ic_cdk::export::serde::Deserialize;
use ic_stable_memory::utils::ic_types::SPrincipal;
use serde::Serialize;

mod upgrader;

#[derive(Default, Serialize, Deserialize, CandidType)]
pub struct CanisterData {
    counter_1: u64,
    counter_2: u64,
    inner_struct: InnerStruct,
    // inner_struct_v2: InnerStructV2,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct InnerStructV2 {
    some_principal: Principal,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct InnerStruct {
    some_principal: SPrincipal,
}

impl Default for InnerStruct {
    fn default() -> Self {
        Self {
            some_principal: SPrincipal(Principal::anonymous()),
        }
    }
}

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
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
