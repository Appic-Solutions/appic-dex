use crate::{
    address::Address,
    cross_chain::{
        rlp_decoder::{Blockchain, RlpDecodeError},
        swap_order::create_swap_order,
        types::Recipient,
    },
    libraries::path_key::Swap,
    minter_client::minter_types::{Minter, MinterKey, SwapOrderCreationError},
    pool::types::{PoolFee, PoolId, PoolState, PoolTickSpacing},
    state::mutate_state,
    validation::swap_args::ValidatedSwapArgs,
};
use candid::Principal;
use ethnum::{I256, U256};
use std::str::FromStr;

pub fn bsc_from_minter() -> MinterKey {
    MinterKey {
        chain_id: 56,
        id: Principal::from_text("2ztvj-yaaaa-aaaap-ahiza-cai").unwrap(),
    }
}
pub fn base_from_minter() -> MinterKey {
    MinterKey {
        chain_id: 8453,
        id: Principal::from_text("4ati2-naaaa-aaaad-qg6la-cai").unwrap(),
    }
}
pub fn prepare_state() {
    mutate_state(|s| {
        s.add_minter(
            bsc_from_minter(),
            Minter {
                twin_usdc_principal: Principal::from_text("z2iye-fyaaa-aaaag-at2pa-cai").unwrap(),
                usdc_address: Address::from_str("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d")
                    .unwrap(),
            },
        );
        s.add_minter(
            base_from_minter(),
            Minter {
                twin_usdc_principal: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                usdc_address: Address::from_str("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")
                    .unwrap(),
            },
        );
        s.set_pool(
            PoolId {
                token0: Principal::from_text("z2iye-fyaaa-aaaag-at2pa-cai").unwrap(),
                token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
                fee: PoolFee(1000),
            },
            PoolState {
                sqrt_price_x96: U256::from(79_383_368_562_352_051_400_232_u128),
                tick: -276_285,
                fee_growth_global_0_x128: U256::ZERO,
                fee_growth_global_1_x128: U256::ZERO,
                liquidity: 98_815_619_458_652_937,
                tick_spacing: PoolTickSpacing(20),
                max_liquidity_per_tick: 3_835_161_415_588_698_631_345_301_964_810_804_u128,
                fee_protocol: 0,
                token0_transfer_fee: U256::from(10_000_000_000_000_000_u128),
                token1_transfer_fee: U256::from(10_000_u128),
                swap_volume0_all_time: U256::ZERO,
                swap_volume1_all_time: U256::ZERO,
                pool_reserve0: U256::from(24_075_968_775_435_293_008_u128),
                pool_reserve1: U256::from(371_340_035_u128),
                generated_swap_fee0: U256::ZERO,
                generated_swap_fee1: U256::ZERO,
            },
        );
        s.set_pool(
            PoolId {
                token0: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
                fee: PoolFee(100),
            },
            PoolState {
                sqrt_price_x96: U256::from(79_348_275_437_447_525_686_522_247_306_u128),
                tick: 30,
                fee_growth_global_0_x128: U256::ZERO,
                fee_growth_global_1_x128: U256::ZERO,
                liquidity: 20_226_441_665,
                tick_spacing: PoolTickSpacing(1),
                max_liquidity_per_tick: 191_757_530_477_355_301_479_181_766_273_477_u128,
                fee_protocol: 0,
                token0_transfer_fee: U256::from(10_000_u128),
                token1_transfer_fee: U256::from(10_000_u128),
                swap_volume0_all_time: U256::ZERO,
                swap_volume1_all_time: U256::ZERO,
                pool_reserve0: U256::from(18_874_016_u128),
                pool_reserve1: U256::from(62_998_876_u128),
                generated_swap_fee0: U256::ZERO,
                generated_swap_fee1: U256::ZERO,
            },
        );
    });
}
#[test]
fn should_create_evm_evm_swap_order() {
    prepare_state();
    let encoded_swap_data = "0xf904948f3937303030303030303030303030309334323534303032363739383931383236333238933431393837303036343530353332333236303084312e3325f90454f901b784383435338f3937303030303030303030303030308734333430393636873433313932363184302e35258633343938373088313136393736323388302e30313933343730f85cf85aaa307834323030303030303030303030303030303030303030303030303030303030303030303030303036aa30783833333538396643443665446236453038663463374333324434663731623534626441303239313383313030c131f90105b901023078303030303030303030303030303030303030303030303030343230303030303030303030303030303030303030303030303030303030303030303030303030363030303030303030303030303030303030303030303030303833333538396663643665646236653038663463376333326434663731623534626461303239313330303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303431653831648a31373538343638323431f8bf8369637087343334303936369334333331323335383335383832333333393631933432393635383539343931393532373533303084302e382530303030f87df83c9b716b7277702d7a696161612d61616161672d6175656d712d6361699b7865766e6d2d67616161612d61616161722d7161666e712d63616983313030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b7a326979652d66796161612d61616161672d61743270612d6361698431303030c0c030f901d682353693343235373038333833353838323333333936319334323534303032363739383931383236333238933431393837303036343530353332333236303084312e3325863334393837308931303030303030303088302e30333637363084302e3035f85cf85aaa307838414337366135316363393530643938323244363862383366453141643937423332436435383064aa30783535643339383332366639393035396646373735343835323436393939303237423331393739353583313030c131f90105b901023078303030303030303030303030303030303030303030303030386163373661353163633935306439383232643638623833666531616439376233326364353830643030303030303030303030303030303030303030303030303535643339383332366639393035396666373735343835323436393939303237623331393739353530303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030336162646166656632373738333039388a31373538343638323432";
    let amount_in = U256::from(4_340_966_u128);
    let token_in_icp_step = Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap();
    let from_minter = base_from_minter(); // Corrected to match data from_chain 8453
    let tx_id = "8453-4-1620328840000";
    let from_address = "0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe"
        .parse::<Address>()
        .unwrap();
    let recipient = "0x000000000000000000000000daf40d6d8fcfbbffd1deba15990b7e08780f7ace";
    match create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        Some(&from_minter),
        tx_id,
        Some(from_address),
        None,
        recipient,
    )
    .unwrap()
    {
        super::types::CrosschainSwapOrder::EvmToEvm {
            tx_id: order_tx_id,
            from_address: order_from_address,
            recipient: order_recipient,
            icp_swap_request,
            evm_swap_step,
            from_minter: order_from_minter,
            to_minter,
        } => {
            assert_eq!(order_tx_id.0, tx_id);
            assert_eq!(order_from_address, from_address);
            assert_eq!(
                order_recipient,
                Recipient::EvmAddress(
                    Address::from_str("0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe").unwrap()
                )
            );
            assert_eq!(order_from_minter, from_minter);
            assert_eq!(to_minter, bsc_from_minter());
            assert_eq!(
                icp_swap_request.token_in(),
                Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap()
            );
            assert_eq!(
                icp_swap_request.token_out(),
                Principal::from_text("z2iye-fyaaa-aaaag-at2pa-cai").unwrap()
            );
            // Add more asserts for icp_swap_request
            assert_eq!(
                icp_swap_request,
                ValidatedSwapArgs::ExactInput {
                    path: vec![
                        Swap {
                            pool_id: PoolId {
                                token0: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai")
                                    .unwrap(),
                                token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai")
                                    .unwrap(),
                                fee: PoolFee(100),
                            },
                            zero_for_one: true,
                        },
                        Swap {
                            pool_id: PoolId {
                                token0: Principal::from_text("z2iye-fyaaa-aaaag-at2pa-cai")
                                    .unwrap(),
                                token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai")
                                    .unwrap(),
                                fee: PoolFee(1000),
                            },
                            zero_for_one: false,
                        },
                    ],
                    amount_in: I256::from(4340966i64),
                    amount_out_minimum: I256::from_str("4296585949195275300").unwrap(),
                    from_subaccount: None,
                    token_in: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                    token_out: Principal::from_text("z2iye-fyaaa-aaaag-at2pa-cai").unwrap(),
                }
            );
            // Assert evm_swap_step fields
            assert_eq!(evm_swap_step.chain_id, Blockchain::Evm(56));
            assert_eq!(
                evm_swap_step.amount_in,
                U256::from_str_radix("4257083835882333961", 10).unwrap()
            );
            // Add more if needed
        }
        _ => panic!("Invalid swap order"),
    }
}
#[test]
fn should_create_evm_to_icp_swap_order() {
    prepare_state();
    let encoded_swap_data = "0xf9024b8f3937303030303030303030303030308734333732333936873433333734313684302e3825f90223f901b684383435338f3937303030303030303030303030308734333630353437873433333837343484302e352586333439383730873334303130393688302e30303533363730f85cf85aaa307834323030303030303030303030303030303030303030303030303030303030303030303030303036aa30783833333538396643443665446236453038663463374333324434663731623534626441303239313383313030c131f90105b901023078303030303030303030303030303030303030303030303030343230303030303030303030303030303030303030303030303030303030303030303030303030363030303030303030303030303030303030303030303030303833333538396663643665646236653038663463376333326434663731623534626461303239313330303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303432333433388a31373538343834323933f8688369637087343336303534378734333732333936873433333734313684302e382530303030f83ef83c9b716b7277702d7a696161612d61616161672d6175656d712d6361699b7865766e6d2d67616161612d61616161722d7161666e712d63616983313030c0c030";
    let amount_in = U256::from(4_360_547_u128);
    let token_in_icp_step = Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap();
    let from_minter = base_from_minter(); // Corrected to match data
    let tx_id = "8453-4-1620328840000";
    let from_address = "0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe"
        .parse::<Address>()
        .unwrap();
    let recipient = "0x1d0b5ef2c95dcfe54bdbeed8236d2101c037c12e4cb0f1e70c6c5bcc03020000";
    match create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        Some(&from_minter),
        tx_id,
        Some(from_address),
        None,
        recipient,
    )
    .unwrap()
    {
        super::types::CrosschainSwapOrder::EvmToIcp {
            tx_id: order_tx_id,
            from_address: order_from_address,
            recipient: order_recipient,
            icp_swap_request,
            from_minter: order_from_minter,
        } => {
            assert_eq!(order_tx_id.0, tx_id);
            assert_eq!(order_from_address, from_address);
            assert_eq!(
                order_recipient,
                Recipient::IcPrincipal(
                    Principal::from_text(
                        "7qi53-mqll3-zmsxo-p4vf5-x3wye-nwsca-oag7a-s4tfq-6htqy-3c3zq-bqe"
                    )
                    .unwrap()
                )
            );
            assert_eq!(order_from_minter, from_minter);
            assert_eq!(
                icp_swap_request.token_in(),
                Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap()
            );
            assert_eq!(
                icp_swap_request.token_out(),
                Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap()
            );
            // Add more asserts for icp_swap_request
            assert_eq!(
                icp_swap_request,
                ValidatedSwapArgs::ExactInputSingle {
                    pool_id: PoolId {
                        token0: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                        token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
                        fee: PoolFee(100),
                    },
                    zero_for_one: true,
                    amount_in: I256::from(4360547i64),
                    amount_out_minimum: I256::from_str("4337416").unwrap(),
                    from_subaccount: None,
                    token_in: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                    token_out: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
                }
            );
        }
        _ => panic!("Invalid swap order"),
    }
}
#[test]
fn should_create_icp_to_evm_swap_order() {
    prepare_state();
    let encoded_swap_data = "0xf9025f87353030303030309031303937373539373034373834353336903130383839373736323731343632353984302e3825f9022df8688369637087353030303030308734393833313435873439363831393584302e332530303030f83ef83c9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d63616983313030c0c030f901c0843834353387343932383834319031303937373539373034373834353336903130383839373736323731343632353984302e382586333439383730873431363434303088302e30303635323930f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c131f90105b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030336531366132333564626338358a31373538343835303331";
    let amount_in = U256::from(4_983_145_u128);
    let token_in_icp_step = Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap();
    let tx_id = "8453-4-1620328840000";
    let recipient = "0x000000000000000000000000daf40d6d8fcfbbffd1deba15990b7e08780f7ace";
    match create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        None,
        tx_id,
        None,
        Some(
            Principal::from_text("7qi53-mqll3-zmsxo-p4vf5-x3wye-nwsca-oag7a-s4tfq-6htqy-3c3zq-bqe")
                .unwrap(),
        ),
        recipient,
    )
    .unwrap()
    {
        super::types::CrosschainSwapOrder::IcpToEvm {
            tx_id: order_tx_id,
            from: order_from,
            recipient: order_recipient,
            icp_swap_request,
            evm_swap_step,
            to_minter,
        } => {
            assert_eq!(order_tx_id.0, tx_id);
            assert_eq!(
                order_from,
                Principal::from_text(
                    "7qi53-mqll3-zmsxo-p4vf5-x3wye-nwsca-oag7a-s4tfq-6htqy-3c3zq-bqe"
                )
                .unwrap()
            );
            assert_eq!(
                order_recipient,
                Recipient::EvmAddress(
                    Address::from_str("0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe").unwrap()
                )
            );
            assert_eq!(to_minter, base_from_minter());
            assert_eq!(
                icp_swap_request.token_in(),
                Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap()
            );
            assert_eq!(
                icp_swap_request.token_out(),
                Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap()
            );
            // Add more asserts for icp_swap_request
            assert_eq!(
                icp_swap_request,
                ValidatedSwapArgs::ExactInputSingle {
                    pool_id: PoolId {
                        token0: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                        token1: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
                        fee: PoolFee(100),
                    },
                    zero_for_one: false,
                    amount_in: I256::from(5000000),
                    amount_out_minimum: I256::from_str("4968195").unwrap(),
                    from_subaccount: None,
                    token_in: Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap(),
                    token_out: Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap(),
                }
            );
            // Assert evm_swap_step
            assert_eq!(evm_swap_step.chain_id, Blockchain::Evm(8453));
            // Add more if needed
        }
        _ => panic!("Invalid swap order"),
    }
}

#[test]
fn should_fail_if_swap_steps_less_than_2() {
    prepare_state();
    let encoded_swap_data = "0xd28431303030833930308338303083312e30c0";
    let amount_in = U256::from(1000u32);
    let token_in_icp_step = Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap();
    let from_minter = base_from_minter();
    let tx_id = "8453-4-1620328840000";
    let from_address = "0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe"
        .parse::<Address>()
        .unwrap();
    let recipient = "0x000000000000000000000000daf40d6d8fcfbbffd1deba15990b7e08780f7ace";
    let result = create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        Some(&from_minter),
        tx_id,
        Some(from_address),
        None,
        recipient,
    );
    assert_eq!(
        result,
        Err(SwapOrderCreationError::InvalidRlpData(
            RlpDecodeError::MissingField
        ))
    );
}

#[test]
fn should_fail_if_origin_and_destination_same() {
    prepare_state();
    let encoded_swap_data = "0xf87f8431303030833930308338303083312e30f86cf584383435338431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030f584383435338431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030";
    let amount_in = U256::from(1000u32);
    let token_in_icp_step = Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap();
    let from_minter = base_from_minter();
    let tx_id = "8453-4-1620328840000";
    let from_address = "0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe"
        .parse::<Address>()
        .unwrap();
    let recipient = "0x000000000000000000000000daf40d6d8fcfbbffd1deba15990b7e08780f7ace";
    let result = create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        Some(&from_minter),
        tx_id,
        Some(from_address),
        None,
        recipient,
    );
    assert_eq!(
        result,
        Err(SwapOrderCreationError::InvalidOriginAndDestinationChain)
    );
}

#[test]
fn should_fail_if_icp_to_evm_with_more_steps() {
    prepare_state();
    let encoded_swap_data = "0xf8b48431303030833930308338303083312e30f8a1f4836963708431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030f584383435338431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030f584383435338431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030";
    let amount_in = U256::from(1000u32);
    let token_in_icp_step = Principal::from_text("xevnm-gaaaa-aaaar-qafnq-cai").unwrap();
    let tx_id = "8453-4-1620328840000";
    let from_principal =
        Principal::from_text("7qi53-mqll3-zmsxo-p4vf5-x3wye-nwsca-oag7a-s4tfq-6htqy-3c3zq-bqe")
            .unwrap();
    let recipient = "0x000000000000000000000000daf40d6d8fcfbbffd1deba15990b7e08780f7ace";
    let result = create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        None,
        tx_id,
        None,
        Some(from_principal),
        recipient,
    );
    assert_eq!(result, Err(SwapOrderCreationError::InvalidToChain));
}

#[test]
fn should_fail_if_invalid_chain_id() {
    prepare_state();
    let encoded_swap_data = "0xf87e8431303030833930308338303083312e30f86bf4833939398431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030f584383435338431303030833930308338303083302e358331303082353084302e303184302e3031c0c0c08a32303030303030303030";
    let amount_in = U256::from(1000u32);
    let token_in_icp_step = Principal::from_text("qkrwp-ziaaa-aaaag-auemq-cai").unwrap();
    let from_minter = base_from_minter();
    let tx_id = "8453-4-1620328840000";
    let from_address = "0xdAf40D6d8FCFBbFfd1deBA15990b7e08780F7ACe"
        .parse::<Address>()
        .unwrap();
    let recipient = "0x000000000000000000000000daf40d6d8fcfbbffd1deba15990b7e08780f7ace";
    let result = create_swap_order(
        encoded_swap_data,
        amount_in,
        token_in_icp_step,
        Some(&from_minter),
        tx_id,
        Some(from_address),
        None,
        recipient,
    );
    assert_eq!(
        result,
        Err(SwapOrderCreationError::InvalidRlpData(
            RlpDecodeError::InvalidChainId("Evm(999)".to_string())
        ))
    );
}
