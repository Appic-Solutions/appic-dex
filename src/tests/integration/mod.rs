// Pocket ic helpers:
// This mod was built by the purpose of simulating the minter_canisters opration on a subnet and testing
// both the deposit and the withdrawal flow to make sure there will be no point of failure in the mentioned flows
// and concurrent requests;

const LEDGER_WASM_BYTES: &[u8] = include_bytes!("./wasm/ledger_canister_u256.wasm.gz");
const APPIC_DEX_WASM_BYTES: &[u8] = include_bytes!("./wasm/appic_dex.wasm");

const TWENTY_TRILLIONS: u64 = 20_000_000_000_000;

const TOKEN_TRANSFER_FEE: u128 = 10_000_000_000_000_u128;

const TWO_HUNDRED_ETH: u128 = 200_000_000_000_000_000_000_u128;

pub mod modify_liquidity;
pub mod swap_tests;

use candid::{CandidType, Nat, Principal};
use ethnum::U256;
use ic_icrc1_ledger::FeatureFlags as LedgerFeatureFlags;
use icrc_ledger_types::{
    icrc1::{
        account::Account as LedgerAccount,
        transfer::{TransferArg, TransferError},
    },
    icrc2::{
        allowance::{Allowance, AllowanceArgs},
        approve::{ApproveArgs, ApproveError},
    },
};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};

use ic_icrc1_ledger::{ArchiveOptions, InitArgs as LedgerInitArgs, LedgerArgument};

use crate::{
    candid_types::{
        pool::{CandidPoolId, CreatePoolArgs, CreatePoolError},
        position::{MintPositionArgs, MintPositionError},
        UserBalanceArgs,
    },
    libraries::{safe_cast::u256_to_nat, sqrt_price_math::tests::SQRT_PRICE_1_1},
};

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

pub fn liquidity_provider_principal() -> Principal {
    Principal::from_text("jswz3-jv6su-cmo3o-izfod-sofls-yng6u-rkxyb-5kyxn-4ughb-ls52c-kae").unwrap()
}

pub fn minting_principal() -> Principal {
    Principal::from_text("2ztvj-yaaaa-aaaap-ahiza-cai").unwrap()
}

pub fn appic_dex_canister_id() -> Principal {
    Principal::from_text("aboy3-giaaa-aaaar-aaaaq-cai").unwrap()
}

fn create_appic_dex_canister(pic: &PocketIc) -> Principal {
    pic.create_canister_with_id(Some(sender_principal()), None, appic_dex_canister_id())
        .expect("Should create the canister")
}

fn install_appic_dex_canister(pic: &PocketIc, canister_id: Principal) {
    pic.install_canister(
        canister_id,
        APPIC_DEX_WASM_BYTES.to_vec(),
        encode_call_args(()).unwrap(),
        Some(sender_principal()),
    );
}

pub fn token0_principal() -> Principal {
    Principal::from_text("zjydy-zyaaa-aaaaj-qnfka-cai").unwrap()
}

pub fn token1_principal() -> Principal {
    Principal::from_text("n44gr-qyaaa-aaaam-qbuha-cai").unwrap()
}

pub fn token2_principal() -> Principal {
    Principal::from_text("ghsi2-tqaaa-aaaan-aaaca-cai").unwrap()
}

pub fn token3_principal() -> Principal {
    Principal::from_text("eysav-tyaaa-aaaap-akqfq-cai").unwrap()
}

fn create_token0_canister(pic: &PocketIc) -> Principal {
    pic.create_canister_with_id(Some(sender_principal()), None, token0_principal())
        .expect("Should create the canister")
}

fn create_token1_canister(pic: &PocketIc) -> Principal {
    pic.create_canister_with_id(Some(sender_principal()), None, token1_principal())
        .expect("Should create the canister")
}

fn create_token2_canister(pic: &PocketIc) -> Principal {
    pic.create_canister_with_id(Some(sender_principal()), None, token2_principal())
        .expect("Should create the canister")
}

fn create_token3_canister(pic: &PocketIc) -> Principal {
    pic.create_canister_with_id(Some(sender_principal()), None, token3_principal())
        .expect("Should create the canister")
}

fn install_token0_canister(pic: &PocketIc, canister_id: Principal) {
    const LEDGER_FEE_SUBACCOUNT: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0x0f, 0xee,
    ];
    const MAX_MEMO_LENGTH: u16 = 80;
    const ICRC2_FEATURE: LedgerFeatureFlags = LedgerFeatureFlags { icrc2: true };

    const THREE_GIGA_BYTES: u64 = 3_221_225_472;

    let ledger_init_bytes = LedgerArgument::Init(LedgerInitArgs {
        minting_account: LedgerAccount::from(minting_principal()),
        fee_collector_account: Some(LedgerAccount {
            owner: Principal::from_slice(&[10]),
            subaccount: Some(LEDGER_FEE_SUBACCOUNT),
        }),
        initial_balances: vec![],
        transfer_fee: Nat::from(10_000_000_000_000_u128),
        decimals: Some(18_u8),
        token_name: "icUSDT.bsc".to_string(),
        token_symbol: "icUSDT.bsc".to_string(),
        metadata: vec![],
        archive_options: ArchiveOptions {
            trigger_threshold: 2_000,
            num_blocks_to_archive: 1_000,
            node_max_memory_size_bytes: Some(THREE_GIGA_BYTES),
            max_message_size_bytes: None,
            controller_id: Principal::from_text("kmcdp-4yaaa-aaaag-ats3q-cai")
                .unwrap()
                .into(),
            more_controller_ids: Some(vec![sender_principal().into()]),
            cycles_for_archive_creation: Some(2_000_000_000_000_u64),
            max_transactions_per_response: None,
        },
        max_memo_length: Some(MAX_MEMO_LENGTH),
        feature_flags: Some(ICRC2_FEATURE),
    });
    pic.install_canister(
        canister_id,
        LEDGER_WASM_BYTES.to_vec(),
        encode_call_args(ledger_init_bytes).unwrap(),
        Some(sender_principal()),
    );
}

fn install_token1_canister(pic: &PocketIc, canister_id: Principal) {
    const LEDGER_FEE_SUBACCOUNT: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0x0f, 0xee,
    ];
    const MAX_MEMO_LENGTH: u16 = 80;
    const ICRC2_FEATURE: LedgerFeatureFlags = LedgerFeatureFlags { icrc2: true };

    const THREE_GIGA_BYTES: u64 = 3_221_225_472;

    let ledger_init_bytes = LedgerArgument::Init(LedgerInitArgs {
        minting_account: LedgerAccount::from(minting_principal()),
        fee_collector_account: Some(LedgerAccount {
            owner: Principal::from_slice(&[10]),
            subaccount: Some(LEDGER_FEE_SUBACCOUNT),
        }),
        initial_balances: vec![],
        transfer_fee: Nat::from(10_000_000_000_000_u128),
        decimals: Some(18_u8),
        token_name: "icUSDT.arb".to_string(),
        token_symbol: "icUSDT.arb".to_string(),
        metadata: vec![],
        archive_options: ArchiveOptions {
            trigger_threshold: 2_000,
            num_blocks_to_archive: 1_000,
            node_max_memory_size_bytes: Some(THREE_GIGA_BYTES),
            max_message_size_bytes: None,
            controller_id: Principal::from_text("kmcdp-4yaaa-aaaag-ats3q-cai")
                .unwrap()
                .into(),
            more_controller_ids: Some(vec![sender_principal().into()]),
            cycles_for_archive_creation: Some(2_000_000_000_000_u64),
            max_transactions_per_response: None,
        },
        max_memo_length: Some(MAX_MEMO_LENGTH),
        feature_flags: Some(ICRC2_FEATURE),
    });
    pic.install_canister(
        canister_id,
        LEDGER_WASM_BYTES.to_vec(),
        encode_call_args(ledger_init_bytes).unwrap(),
        Some(sender_principal()),
    );
}

fn install_token2_canister(pic: &PocketIc, canister_id: Principal) {
    const LEDGER_FEE_SUBACCOUNT: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0x0f, 0xee,
    ];
    const MAX_MEMO_LENGTH: u16 = 80;
    const ICRC2_FEATURE: LedgerFeatureFlags = LedgerFeatureFlags { icrc2: true };

    const THREE_GIGA_BYTES: u64 = 3_221_225_472;

    let ledger_init_bytes = LedgerArgument::Init(LedgerInitArgs {
        minting_account: LedgerAccount::from(minting_principal()),
        fee_collector_account: Some(LedgerAccount {
            owner: Principal::from_slice(&[10]),
            subaccount: Some(LEDGER_FEE_SUBACCOUNT),
        }),
        initial_balances: vec![],
        transfer_fee: Nat::from(10_000_000_000_000_u128),
        decimals: Some(18_u8),
        token_name: "icUSDT.eth".to_string(),
        token_symbol: "icUSDT.eth".to_string(),
        metadata: vec![],
        archive_options: ArchiveOptions {
            trigger_threshold: 2_000,
            num_blocks_to_archive: 1_000,
            node_max_memory_size_bytes: Some(THREE_GIGA_BYTES),
            max_message_size_bytes: None,
            controller_id: Principal::from_text("kmcdp-4yaaa-aaaag-ats3q-cai")
                .unwrap()
                .into(),
            more_controller_ids: Some(vec![sender_principal().into()]),
            cycles_for_archive_creation: Some(2_000_000_000_000_u64),
            max_transactions_per_response: None,
        },
        max_memo_length: Some(MAX_MEMO_LENGTH),
        feature_flags: Some(ICRC2_FEATURE),
    });
    pic.install_canister(
        canister_id,
        LEDGER_WASM_BYTES.to_vec(),
        encode_call_args(ledger_init_bytes).unwrap(),
        Some(sender_principal()),
    );
}

fn install_token3_canister(pic: &PocketIc, canister_id: Principal) {
    const LEDGER_FEE_SUBACCOUNT: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0x0f, 0xee,
    ];
    const MAX_MEMO_LENGTH: u16 = 80;
    const ICRC2_FEATURE: LedgerFeatureFlags = LedgerFeatureFlags { icrc2: true };

    const THREE_GIGA_BYTES: u64 = 3_221_225_472;

    let ledger_init_bytes = LedgerArgument::Init(LedgerInitArgs {
        minting_account: LedgerAccount::from(minting_principal()),
        fee_collector_account: Some(LedgerAccount {
            owner: Principal::from_slice(&[10]),
            subaccount: Some(LEDGER_FEE_SUBACCOUNT),
        }),
        initial_balances: vec![],
        transfer_fee: Nat::from(10_000_000_000_000_u128),
        decimals: Some(18_u8),
        token_name: "icUSDT.arb".to_string(),
        token_symbol: "icUSDT.arb".to_string(),
        metadata: vec![],
        archive_options: ArchiveOptions {
            trigger_threshold: 2_000,
            num_blocks_to_archive: 1_000,
            node_max_memory_size_bytes: Some(THREE_GIGA_BYTES),
            max_message_size_bytes: None,
            controller_id: Principal::from_text("kmcdp-4yaaa-aaaag-ats3q-cai")
                .unwrap()
                .into(),
            more_controller_ids: Some(vec![sender_principal().into()]),
            cycles_for_archive_creation: Some(2_000_000_000_000_u64),
            max_transactions_per_response: None,
        },
        max_memo_length: Some(MAX_MEMO_LENGTH),
        feature_flags: Some(ICRC2_FEATURE),
    });
    pic.install_canister(
        canister_id,
        LEDGER_WASM_BYTES.to_vec(),
        encode_call_args(ledger_init_bytes).unwrap(),
        Some(sender_principal()),
    );
}

pub fn mint_tokens(pic: &PocketIc, token: Principal) {
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: sender_principal().into(),
        fee: None,
        created_at_time: None,
        memo: None,
        amount: Nat::from(410_000_000_000_000_000_000_u128), // 410 eth
    };

    let _result = update_call::<TransferArg, Result<Nat, TransferError>>(
        pic,
        token,
        "icrc1_transfer",
        transfer_args,
        Some(minting_principal()),
    )
    .unwrap();

    let transfer_args_to_liquidity_provider = TransferArg {
        from_subaccount: None,
        to: liquidity_provider_principal().into(),
        fee: None,
        created_at_time: None,
        memo: None,
        amount: Nat::from(410_000_000_000_000_000_000_u128), // 410 eth
    };

    let _result = update_call::<TransferArg, Result<Nat, TransferError>>(
        pic,
        token,
        "icrc1_transfer",
        transfer_args_to_liquidity_provider,
        Some(minting_principal()),
    )
    .unwrap();
}

pub fn five_ticks(pic: &PocketIc) {
    pic.tick();
    pic.tick();
    pic.tick();
    pic.tick();
    pic.tick();
}

pub fn create_and_install_canisters(pic: &PocketIc) {
    // Create and install token0 ledger
    let token0_canister_id = create_token0_canister(&pic);
    pic.add_cycles(token0_canister_id, TWENTY_TRILLIONS.into());
    install_token0_canister(&pic, token0_canister_id);
    five_ticks(&pic);

    // Create and install token1 ledger
    let token1_canister_id = create_token1_canister(&pic);
    pic.add_cycles(token1_canister_id, TWENTY_TRILLIONS.into());
    install_token1_canister(&pic, token1_canister_id);
    five_ticks(&pic);

    // Create and install token1 ledger
    let token2_canister_id = create_token2_canister(&pic);
    pic.add_cycles(token2_canister_id, TWENTY_TRILLIONS.into());
    install_token2_canister(&pic, token2_canister_id);
    five_ticks(&pic);

    // Create and install token1 ledger
    let token3_canister_id = create_token3_canister(&pic);
    pic.add_cycles(token3_canister_id, TWENTY_TRILLIONS.into());
    install_token3_canister(&pic, token3_canister_id);
    five_ticks(&pic);

    // mint tokens
    mint_tokens(&pic, token0_canister_id);
    mint_tokens(&pic, token1_canister_id);
    mint_tokens(&pic, token2_canister_id);
    mint_tokens(&pic, token3_canister_id);

    // Create and install appic_dex
    let appic_dex_canister_id = create_appic_dex_canister(&pic);
    pic.add_cycles(appic_dex_canister_id, TWENTY_TRILLIONS.into());
    install_appic_dex_canister(&pic, appic_dex_canister_id);
    five_ticks(&pic);
    five_ticks(&pic);
}

pub fn create_pool_with_liquidity(pic: &PocketIc, token_0: Principal, token_1: Principal) {
    let create_args = CreatePoolArgs {
        token_a: token_0,
        token_b: token_1,
        fee: Nat::from(3000_u32),
        sqrt_price_x96: u256_to_nat(*SQRT_PRICE_1_1),
    };

    let create_pool_result = update_call::<CreatePoolArgs, Result<CandidPoolId, CreatePoolError>>(
        pic,
        appic_dex_canister_id(),
        "create_pool",
        create_args,
        None,
    );

    println!("{:?}", create_pool_result);

    // Approval Section
    // Calling icrc2_approve and giving the permission to appic_dex for taking funds from users principal
    let _approve_result = update_call::<ApproveArgs, Result<Nat, ApproveError>>(
        &pic,
        token_0,
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
            amount: Nat::from(
                TWO_HUNDRED_ETH + TWO_HUNDRED_ETH, // 400 ethers
            ),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
        None,
    )
    .unwrap();

    // Approval Section
    // Calling icrc2_approve and giving the permission to appic_dex for taking funds from users principal
    let _approve_result = update_call::<ApproveArgs, Result<Nat, ApproveError>>(
        &pic,
        token_1,
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
            amount: Nat::from(
                TWO_HUNDRED_ETH + TWO_HUNDRED_ETH, // 400 ethers
            ),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
        None,
    )
    .unwrap();

    let _balance0 = query_call::<LedgerAccount, Nat>(
        &pic,
        token_0,
        "icrc1_balance_of",
        LedgerAccount::from(sender_principal()),
    );

    let _balance1 = query_call::<LedgerAccount, Nat>(
        &pic,
        token_1,
        "icrc1_balance_of",
        LedgerAccount::from(sender_principal()),
    );

    let _allowance0 = query_call::<AllowanceArgs, Allowance>(
        &pic,
        token_0,
        "icrc2_allowance",
        AllowanceArgs {
            account: LedgerAccount {
                owner: sender_principal(),
                subaccount: None,
            },
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
        },
    );

    let _allowance1 = query_call::<AllowanceArgs, Allowance>(
        &pic,
        token_1,
        "icrc2_allowance",
        AllowanceArgs {
            account: LedgerAccount {
                owner: sender_principal(),
                subaccount: None,
            },
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
        },
    );

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    let _user_balance0 = query_call::<UserBalanceArgs, Nat>(
        &pic,
        appic_dex_canister_id(),
        "user_balance",
        UserBalanceArgs {
            token: token_0,
            user: sender_principal(),
        },
    );
    let _user_balance1 = query_call::<UserBalanceArgs, Nat>(
        &pic,
        appic_dex_canister_id(),
        "user_balance",
        UserBalanceArgs {
            token: token_1,
            user: sender_principal(),
        },
    );

    let mint_args = MintPositionArgs {
        pool: CandidPoolId {
            token0: token_0,
            token1: token_1,
            fee: Nat::from(3000_u32),
        },
        tick_lower: candid::Int::from(-887220),
        tick_upper: candid::Int::from(887220),
        amount0_max: u256_to_nat(U256::from(TWO_HUNDRED_ETH)),
        amount1_max: u256_to_nat(U256::from(TWO_HUNDRED_ETH)),
        from_subaccount: None,
    };

    let _mint_result = update_call::<MintPositionArgs, Result<Nat, MintPositionError>>(
        &pic,
        appic_dex_canister_id(),
        "mint_position",
        mint_args,
        None,
    );
}

pub fn set_up() -> PocketIc {
    let pic = PocketIc::new();

    create_and_install_canisters(&pic);

    five_ticks(&pic);
    five_ticks(&pic);

    create_pool_with_liquidity(&pic, token0_principal(), token1_principal());
    create_pool_with_liquidity(&pic, token1_principal(), token2_principal());
    create_pool_with_liquidity(&pic, token2_principal(), token3_principal());

    pic
}

pub fn get_balance(pic: &PocketIc, token: Principal, user_principal: Principal) -> Nat {
    query_call(
        pic,
        token,
        "icrc1_balance_of",
        LedgerAccount::from(user_principal),
    )
}

#[test]
fn test_init() {
    let pic = PocketIc::new();

    create_and_install_canisters(&pic);

    five_ticks(&pic);
    five_ticks(&pic);

    create_pool_with_liquidity(&pic, token0_principal(), token1_principal());
    create_pool_with_liquidity(&pic, token1_principal(), token2_principal());
    create_pool_with_liquidity(&pic, token2_principal(), token3_principal());
}
