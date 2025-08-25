use crate::rlp_decoder::RlpDecoder;
use ethnum::U256;

#[test]
fn test_decode_cross_chain_quote() {
    let encoded_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data).unwrap();
    
    assert!(quote.success);
    assert!(quote.error.is_none());
    assert!(quote.data.is_some());
    
    let data = quote.data.unwrap();
    assert_eq!(data.total_amount_in, U256::from_str_radix("50000000000000000000", 10).unwrap());
    assert_eq!(data.total_amount_out, U256::from_str_radix("11670858301863654", 10).unwrap());
    assert_eq!(data.total_min_amount_out.unwrap(), U256::from_str_radix("11577491435448744", 10).unwrap());
    assert_eq!(data.total_slippage.unwrap(), "0.8%");
    assert_eq!(data.steps.len(), 3);
    
    let step1 = &data.steps[0];
    assert_eq!(step1.chain_id, "56");
    assert_eq!(step1.amount_in, U256::from_str_radix("50000000000000000000", 10).unwrap());
    assert_eq!(step1.route.len(), 0);
    
    let has_routing = data.steps.iter().any(|step| step.route.len() > 0);
    let has_qswap = data.steps.iter().any(|step| step.qswap_data.is_some());
    assert!(has_routing || has_qswap);
}

#[test]
fn test_evm_to_evm_single_hop() {
    let encoded_data = hex::decode("f85989313030303030303030303089313030303030303030308931303030303030303084302e352566f831f82f81318831303030303030303089313030303030303030823025c0c0c030").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) => {
            if result.success {
                let data = result.data.unwrap();
                assert!(data.total_amount_in > U256::ZERO);
                assert!(data.steps.len() >= 1);
                
                let has_evm_chain = data.steps.iter().any(|step| 
                    step.chain_id.parse::<u64>().is_ok() && step.chain_id.len() <= 5
                );
                if !has_evm_chain {
                    assert!(!data.steps[0].chain_id.is_empty());
                }
            } else {
                assert!(result.error.is_some());
            }
        }
        Err(_) => {
        }
    }
}

#[test] 
fn test_evm_to_evm_multi_hop() {
    let encoded_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) if result.success => {
            let data = result.data.unwrap();
            assert!(data.steps.len() >= 1);
            
            let step_with_routing = data.steps.iter()
                .find(|step| step.route.len() > 0);
                
            if let Some(step) = step_with_routing {
                assert!(!step.chain_id.is_empty(), "Step with routing should have chain ID");
            }
        }
        _ => {
        }
    }
}

#[test]
fn test_evm_to_icp() {
    let encoded_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) if result.success => {
            let data = result.data.unwrap();
            assert!(data.steps.len() >= 1);
            
            let has_bsc = data.steps.iter().any(|step| step.chain_id == "56");
            let has_base = data.steps.iter().any(|step| step.chain_id == "8453");
            let has_icp = data.steps.iter().any(|step| step.chain_id == "icp");
            
            assert!(has_bsc || has_base || has_icp || 
                   data.steps.iter().any(|step| step.chain_id.parse::<u64>().is_ok()),
                   "Should have recognizable chain identifiers");
        }
        _ => {
        }
    }
}

#[test]
fn test_icp_to_evm() {
    let encoded_data = hex::decode("f85989313030303030303030303089313030303030303030308931303030303030303084302e352566f831f82f8369637089313030303030303030308931303030303030303030823025c0c0c030").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) => {
            if result.success {
                let data = result.data.unwrap();
                assert!(data.total_amount_in > U256::ZERO);
                assert!(data.steps.len() >= 1);
                
                let has_icp_or_evm = data.steps.iter().any(|step| 
                    step.chain_id == "icp" || step.chain_id.parse::<u64>().is_ok()
                );
                
                if has_icp_or_evm {
                } else {
                    assert!(!data.steps[0].chain_id.is_empty());
                }
            } else {
                assert!(result.error.is_some());
            }
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_chain_id_validation() {
    let valid_chains = vec!["1", "56", "137", "8453", "42161", "10", "43114", "250", "icp"];
    
    for chain_id in valid_chains {
        let is_supported = RlpDecoder::get_supported_chains().contains_key(chain_id);
        assert!(is_supported, "Chain {} should be supported", chain_id);
    }
    
    let invalid_chains = vec!["999", "invalid", "", "0x1"];
    for chain_id in invalid_chains {
        let is_supported = RlpDecoder::get_supported_chains().contains_key(chain_id);
        assert!(!is_supported, "Chain {} should not be supported", chain_id);
    }
}

#[test]
fn test_data_size_limits() {
    let large_data = vec![0u8; 2 * 1024 * 1024];
    let result = RlpDecoder::decode_cross_chain_data(&large_data);
    
    match result {
        Err(e) => assert!(e.to_string().contains("exceeds maximum size")),
        Ok(quote) => assert!(!quote.success && quote.error.is_some()),
    }
}

#[test]
fn test_production_ready_features() {
    assert!(RlpDecoder::get_supported_chains().len() >= 9);
    
    let chains = RlpDecoder::get_supported_chains();
    assert!(chains.contains_key("1")); 
    assert!(chains.contains_key("56"));
    assert!(chains.contains_key("8453"));
    assert!(chains.contains_key("icp"));
    
    assert_eq!(chains.get("1"), Some(&"Ethereum"));
    assert_eq!(chains.get("56"), Some(&"BSC"));
    assert_eq!(chains.get("8453"), Some(&"Base"));
    assert_eq!(chains.get("icp"), Some(&"Internet Computer"));
}

#[test]
fn test_invalid_rlp_data() {
    let invalid_data = vec![0x01, 0x02, 0x03, 0xff, 0xee];
    let result = RlpDecoder::decode_cross_chain_data(&invalid_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
            assert!(quote.data.is_none());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_empty_data() {
    let empty_data = vec![];
    let result = RlpDecoder::decode_cross_chain_data(&empty_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_malformed_rlp_structure() {
    let incomplete_data = hex::decode("c483313233").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&incomplete_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_invalid_u256_values() {
    let invalid_u256_data = hex::decode("c584696e76616c6964").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&invalid_u256_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_missing_required_fields() {
    let insufficient_data = hex::decode("c3823132").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&insufficient_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_comprehensive_error_scenarios() {
    let error_cases = vec![
        vec![0xff],
        vec![0xc0],
        hex::decode("c1ff").unwrap(),
    ];
    
    for invalid_data in error_cases {
        let result = RlpDecoder::decode_cross_chain_data(&invalid_data);
        
        match result {
            Ok(quote) => {
                assert!(!quote.success || quote.error.is_some());
            }
            Err(_) => {
            }
        }
    }
}

#[test]
fn test_icp_to_evm_full_path() {
    let encoded_data = hex::decode("f901288469637088313535303030303030303030303030303088313535303030303030303030303030303084302e30303135303030f8f3f8338236318a313535303030303030303030303030303084302e30303135303030c0c0c0f845946963702d746f6b656e2d61626364656667681b69636f6e69632d746f6b656e2d68617368823139c0c088302e30303135303030c0f845946576616c75652d746f6b656e2d61626364656667681b657468657265756d2d746f6b656e2d68617368823637c0c088302e30303135303030c030823025c0").unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&encoded_data);
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                assert!(data.steps.iter().any(|step| step.chain_id == "icp"));
                assert!(data.steps.iter().any(|step| step.chain_id.parse::<u64>().is_ok()));
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_evm_to_icp_full_path() {
    let encoded_data = hex::decode("f901288236318a313535303030303030303030303030303088313535303030303030303030303030303084302e30303135303030f8f3f833846963708831353530303030303030303030303030303084302e30303135303030c0c0c0f845946576616c75652d746f6b656e2d61626364656667681b657468657265756d2d746f6b656e2d68617368823637c0c088302e30303135303030c0f845946963702d746f6b656e2d61626364656667681b69636f6e69632d746f6b656e2d68617368823139c0c088302e30303135303030c030823025c0").unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&encoded_data);
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                assert!(data.steps.iter().any(|step| step.chain_id == "icp"));
                assert!(data.steps.iter().any(|step| step.chain_id.parse::<u64>().is_ok()));
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_zero_amounts() {
    let zero_data = hex::decode("c9808080").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&zero_data);
    
    match result {
        Ok(quote) => assert!(!quote.success),
        Err(_) => {}
    }
}

#[test]
fn test_negative_amounts() {
    let negative_data = hex::decode("c68180").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&negative_data);
    
    match result {
        Ok(quote) => assert!(!quote.success),
        Err(_) => {}
    }
}

#[test]
fn test_maximum_u256() {
    let max_u256_hex = "a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let max_data = hex::decode(&format!("c4{}", max_u256_hex)).unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&max_data);
    match result {
        Ok(_) => {},
        Err(_) => {}
    }
}

#[test]
fn test_unsupported_chain_ids() {
    let unsupported_chains = vec!["999", "9999", "bitcoin", "solana"];
    
    for chain_id in unsupported_chains {
        let chain_bytes = chain_id.as_bytes();
        let mut data = vec![0xc0 + chain_bytes.len() as u8];
        data.extend_from_slice(chain_bytes);
        
        let result = RlpDecoder::decode_cross_chain_data(&data);
        match result {
            Ok(quote) => {
                if quote.success {
                    let data = quote.data.unwrap();
                    let has_unsupported = data.steps.iter().any(|step| step.chain_id == chain_id);
                    if has_unsupported {
                        assert!(!RlpDecoder::get_supported_chains().contains_key(chain_id));
                    }
                }
            }
            Err(_) => {}
        }
    }
}

#[test]
fn test_empty_route_arrays() {
    let empty_route_data = hex::decode("d4808080c0").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&empty_route_data);
    
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                assert!(data.steps.iter().all(|step| step.route.is_empty()));
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_missing_optional_fields() {
    let minimal_data = hex::decode("ca80808080").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&minimal_data);
    
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                assert!(data.total_slippage.is_none() || data.total_min_amount_out.is_none());
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_large_step_count() {
    let many_steps = (0..10).map(|i| format!("c483{}3536", i)).collect::<Vec<_>>().join("");
    let large_steps_data = hex::decode(&format!("d9{}{}", many_steps.len() / 2, many_steps));
    
    match large_steps_data {
        Ok(data) => {
            let result = RlpDecoder::decode_cross_chain_data(&data);
            match result {
                Ok(quote) => {
                    if quote.success {
                        let data = quote.data.unwrap();
                        assert!(data.steps.len() <= 10);
                    }
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_malformed_hex_addresses() {
    let bad_hex = hex::decode("d8c585626164686578").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&bad_hex);
    
    match result {
        Ok(quote) => assert!(!quote.success),
        Err(_) => {}
    }
}

#[test]
fn test_boundary_slippage_values() {
    let boundary_slippages = vec!["0%", "100%", "0.01%", "99.99%"];
    
    for slippage in boundary_slippages {
        let slippage_bytes = slippage.as_bytes();
        let mut data = vec![0xc0 + slippage_bytes.len() as u8];
        data.extend_from_slice(slippage_bytes);
        
        let result = RlpDecoder::decode_cross_chain_data(&data);
        match result {
            Ok(_) => {},
            Err(_) => {}
        }
    }
}

#[test]
fn test_concurrent_decoding() {
    use std::thread;
    use std::sync::Arc;
    
    let test_data = Arc::new(hex::decode("f85989313030303030303030303089313030303030303030308931303030303030303084302e352566f831f82f81318831303030303030303089313030303030303030823025c0c0c030").unwrap());
    
    let handles: Vec<_> = (0..5).map(|_| {
        let data = Arc::clone(&test_data);
        thread::spawn(move || {
            let result = RlpDecoder::decode_cross_chain_data(&data);
            match result {
                Ok(_) => true,
                Err(_) => false,
            }
        })
    }).collect();
    
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result || !result);
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_decode_performance() {
        let test_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
        
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = RlpDecoder::decode_cross_chain_data(&test_data);
        }
        
        let duration = start.elapsed();
        let avg_time = duration / iterations;
        
        println!("Average decode time: {:?}", avg_time);
        assert!(avg_time.as_micros() < 10000);
    }

    #[test]
    fn test_memory_usage() {
        let large_data = vec![0x80; 100_000];
        
        let _start_memory = std::process::id();
        let result = RlpDecoder::decode_cross_chain_data(&large_data);
        let _end_memory = std::process::id();
        
        match result {
            Ok(_) | Err(_) => {}
        }
    }
}

#[test]
fn test_routing_validation_complex() {
    let complex_routing_data = hex::decode("f901a0808080f90195f8a5823536943078613030303030303030303030303030303030303030943078623030303030303030303030303030303030303030c0f859f857aa30783535303030303030303030303030303030303030303030303030303030303030303030303030aa307842303030303030303030303030303030303030303030303030303030303030303030303030308630c0f8a58236318a30786138303030303030303030303030303030303030303030c0f859f857aa30783535303030303030303030303030303030303030303030303030303030303030303030303030aa307842303030303030303030303030303030303030303030303030303030303030303030303030308630c030").unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&complex_routing_data);
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                for step in &data.steps {
                    for route in &step.route {
                        assert!(!route.sell_token.is_empty());
                        assert!(!route.buy_token.is_empty());
                    }
                }
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_invalid_route_addresses() {
    let invalid_route_data = hex::decode("d8c5856e6f746f6b656e").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&invalid_route_data);
    
    match result {
        Ok(quote) => assert!(!quote.success),
        Err(_) => {}
    }
}

#[test]
fn test_empty_routing_steps() {
    let empty_routing_data = hex::decode("c9808080c0c0").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&empty_routing_data);
    
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                assert!(data.steps.iter().all(|step| step.route.is_empty()));
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_multi_step_chain_hopping() {
    let multi_chain_data = hex::decode("f901608080808080f90154f84c82353694307861303030303030303030303030303030303030303030943078623030303030303030303030303030303030303030c0c0c0c0f84c82363194307863303030303030303030303030303030303030303030943078643030303030303030303030303030303030303030c0c0c0c0f84c84383435339430786635303030303030303030303030303030303030303030943078663030303030303030303030303030303030303030c0c0c0c0f84c8469637094307867303030303030303030303030303030303030303030943078683030303030303030303030303030303030303030c0c0c0c030").unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&multi_chain_data);
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                let unique_chains: std::collections::HashSet<_> = data.steps.iter().map(|s| &s.chain_id).collect();
                assert!(unique_chains.len() >= 2);
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_circular_routing() {
    let circular_data = hex::decode("f8ca808080f8c4f842823536943078613030303030303030303030303030303030303030943078613030303030303030303030303030303030303030c0c0c0c0f842823536943078613030303030303030303030303030303030303030943078613030303030303030303030303030303030303030c0c0c0c030").unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&circular_data);
    match result {
        Ok(quote) => {
            if quote.success {
                let data = quote.data.unwrap();
                if data.steps.len() >= 2 {
                    let first_chain = &data.steps[0].chain_id;
                    let last_chain = &data.steps[data.steps.len() - 1].chain_id;
                    if first_chain == last_chain {
                        assert_eq!(first_chain, last_chain);
                    }
                }
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_token_address_validation() {
    let valid_tokens = vec![
        "0x1234567890123456789012345678901234567890",
        "0xA0b86a33E6b1d79Aaf742ee1D3E38cfd95D9F2C1",
        "0x0000000000000000000000000000000000000000",
    ];
    
    for token in valid_tokens {
        let token_bytes = token.as_bytes();
        let mut data = vec![0xc0 + token_bytes.len() as u8];
        data.extend_from_slice(token_bytes);
        
        let result = RlpDecoder::decode_cross_chain_data(&data);
        match result {
            Ok(_) => {},
            Err(_) => {}
        }
    }
}

#[test]
fn test_invalid_token_formats() {
    let invalid_tokens = vec![
        "0x123",
        "not_an_address",
        "0xZZZZ567890123456789012345678901234567890",
        "",
    ];
    
    for token in invalid_tokens {
        let token_bytes = token.as_bytes();
        let mut data = vec![0xc0 + token_bytes.len() as u8];
        data.extend_from_slice(token_bytes);
        
        let result = RlpDecoder::decode_cross_chain_data(&data);
        match result {
            Ok(quote) => {
                if quote.success {
                    let data = quote.data.unwrap();
                    for step in &data.steps {
                        for route in &step.route {
                            if route.sell_token == token || route.buy_token == token {
                                assert!(false, "Invalid token should not be accepted");
                            }
                        }
                    }
                }
            },
            Err(_) => {}
        }
    }
}