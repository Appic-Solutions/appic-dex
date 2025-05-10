// Pocket ic helpers:
// This mod was built by the purpose of simulating the minter_canisters opration on a subnet and testing
// both the deposit and the withdrawal flow to make sure there will be no point of failure in the mentioned flows
// and concurrent requests;

const LEDGER_WASM_BYTES: &[u8] = include_bytes!("./wasm/ledger_canister_u256.wasm.gz");

const TWENTY_TRILLIONS: u64 = 20_000_000_000_000;

const FIVE_TRILLIONS: u64 = 5_000_000_000_000;

const FOUR_TRILLIONS: u64 = 4_000_000_000_000;

const TWO_TRILLIONS: u64 = 2_000_000_000_000;

use candid::{CandidType, Nat, Principal};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};

use ic_icrc1_ledger::{ArchiveOptions, InitArgs as LedgerInitArgs, LedgerArgument};

pub fn query_call<I, O>(pic: &PocketIc, canister_id: Principal, method: &str, payload: I) -> O
where
    O: CandidType + for<'a> serde::Deserialize<'a>,
    I: CandidType,
{
    let wasm_result = pic
        .query_call(
            canister_id,
            sender_principal(),
            method,
            encode_call_args(payload).unwrap(),
        )
        .unwrap();

    decode_wasm_result::<O>(wasm_result).unwrap()
}

pub fn update_call<I, O>(
    pic: &PocketIc,
    canister_id: Principal,
    method: &str,
    payload: I,
    sender: Option<Principal>,
) -> O
where
    O: CandidType + for<'a> serde::Deserialize<'a>,
    I: CandidType,
{
    let sender_principal = match sender {
        Some(p_id) => p_id,
        None => sender_principal(),
    };
    let wasm_result = pic
        .update_call(
            canister_id,
            sender_principal,
            method,
            encode_call_args(payload).unwrap(),
        )
        .unwrap();

    decode_wasm_result::<O>(wasm_result).unwrap()
}

pub fn encode_call_args<I>(args: I) -> Result<Vec<u8>, ()>
where
    I: CandidType,
{
    Ok(candid::encode_one(args).unwrap())
}

pub fn decode_wasm_result<O>(wasm_result: WasmResult) -> Result<O, ()>
where
    O: CandidType + for<'a> serde::Deserialize<'a>,
{
    match wasm_result {
        pocket_ic::WasmResult::Reply(vec) => Ok(candid::decode_one(&vec).unwrap()),
        pocket_ic::WasmResult::Reject(_) => Err(()),
    }
}

pub fn create_pic() -> PocketIc {
    PocketIcBuilder::new()
        .with_nns_subnet()
        .with_ii_subnet()
        .with_application_subnet()
        .build()
}

pub fn sender_principal() -> Principal {
    Principal::from_text("matbl-u2myk-jsllo-b5aw6-bxboq-7oon2-h6wmo-awsxf-pcebc-4wpgx-4qe").unwrap()
}
