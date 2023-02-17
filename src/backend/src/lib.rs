use std::cell::RefCell;

use candid::export_service;
use ic_cdk::{
    export::{candid::CandidType, serde::Deserialize},
    storage,
};

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    counter_1: u64,
}

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterDataV2 {
    counter_1: u128,
}

thread_local! {
    // static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
    // static CANISTER_DATA_V2: RefCell<CanisterDataV2> = RefCell::default();
    static CANISTER_DATA_V2: RefCell<CanisterDataV2> = RefCell::default();
}

// #[ic_cdk::query]
// #[candid::candid_method(query)]
// fn get_counter_1() -> u64 {
//     CANISTER_DATA.with(|canister_data_ref_cell| canister_data_ref_cell.borrow().counter_1)
// }

// #[ic_cdk::update]
// #[candid::candid_method(update)]
// fn increment_counter_1() {
//     CANISTER_DATA.with(|canister_data_ref_cell| {
//         canister_data_ref_cell.borrow_mut().counter_1 += 1;
//     });
// }

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    // CANISTER_DATA.with(|canister_data_ref_cell| {
    //     let canister_data = canister_data_ref_cell.take();

    //     storage::stable_save((canister_data,)).ok();
    // });
    CANISTER_DATA_V2.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();

        storage::stable_save((canister_data,)).ok();
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // match storage::stable_restore() {
    //     Ok((canister_data,)) => {
    //         CANISTER_DATA.with(|canister_data_ref_cell| {
    //             *canister_data_ref_cell.borrow_mut() = canister_data;
    //         });
    //     }
    //     Err(e) => {
    //         ic_cdk::print(format!("Error: {:?}", e));
    //         panic!("Failed to restore canister data from stable memory");
    //     }
    // }
    match storage::stable_restore::<(CanisterDataV2,)>() {
        Ok((canister_data,)) => {
            CANISTER_DATA_V2.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(_) => match storage::stable_restore::<(CanisterData,)>() {
            Ok((canister_data,)) => {
                let canister_data_v2 = CanisterDataV2 {
                    counter_1: canister_data.counter_1 as u128,
                };
                CANISTER_DATA_V2.with(|canister_data_ref_cell| {
                    *canister_data_ref_cell.borrow_mut() = canister_data_v2;
                });
            }
            Err(_) => panic!("Failed to decode with both V1 and V2"),
        },
    }
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
