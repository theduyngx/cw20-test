/*
Testing for contract and state.
*/

#[cfg(test)]
mod tests {
    use crate::contract::*;
    use crate::error::ContractError;
    use crate::msg::{
        InstantiateMsg, CreateMsg, ExecuteMsg, QueryMsg, ReceiveMsg,
        ListResponse, DetailsResponse, BalanceHuman
    };

    use sha2::{Digest, Sha256};
    use cosmwasm_std::{
        coins, from_binary, to_binary, StdError, Uint128,
        Timestamp, BankMsg, Env, SubMsg, WasmMsg
    };
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info
    };
    use cw20::{
        Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration
    };

    /// Preimage - the default testing hash input
    fn preimage() -> String {
        hex::encode(b"This is a string, 32 bytes long.")
    }

    /// The custom preimage
    fn custom_preimage(int: u16) -> String {
        hex::encode(format!("This is a custom string: {:>7}", int))
    }

    /// Default hashed of the preimage
    fn real_hash() -> String {
        hex::encode(&Sha256::digest(&hex::decode(preimage()).unwrap()))
    }

    /// Hashed of the custom preimage
    fn custom_hash(int: u16) -> String {
        hex::encode(&Sha256::digest(&hex::decode(custom_preimage(int)).unwrap()))
    }

    /// Mock block height within the chain
    fn mock_env_height(height: u64) -> Env {
        let mut env = mock_env();
        env.block.height = height;
        env
    }

    /// Test printing binary format (or more accurately the base64) of a Create Message to be used for the
    /// Receive Cw20 Message.
    #[test]
    #[ignore = "get_binary"]
    pub fn print_binary() {
        let create_msg = CreateMsg {
            id: "some_id".to_string(),
            hash: "4d9dbecbaaf42653d09a95c7e1986a047ce98afab5f9f8a4f98b20aa9913c984".to_string(),
            recipient: "orai1tcenqk4f26vdz97ewdfcefr3akntzghxj7gcaw".to_string(),
            expires: Expiration::AtHeight(22222222),
        };
        let msg = ReceiveMsg::Create(create_msg);
        println!("\n{}\n", to_binary(&msg).unwrap())
    }


    /// ------------------------ Contract tests ------------------------ ///

    mod contract_test {
        use super::*;

        /// Instantiate test - due to its simplicity, only checksum-ing the response
        #[test]
        fn test_instantiate() {
            let mut deps = mock_dependencies();

            // Instantiate an empty contract
            let instantiate_msg = InstantiateMsg {};
            let info = mock_info("anyone", &[]);
            let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
            assert_eq!(0, res.messages.len());
        }

        /// Test create
        #[test]
        fn test_create() {
            let mut deps = mock_dependencies();

            let info = mock_info("anyone", &[]);
            instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();

            let sender = String::from("sender0001");
            let balance = coins(100, "tokens");

            // Cannot create, invalid ids
            let info = mock_info(&sender, &balance);
            for id in &["sh", "atomic_swap_id_too_long"] {
                let create = CreateMsg {
                    id: id.to_string(),
                    hash: real_hash(),
                    recipient: String::from("rcpt0001"),
                    expires: Expiration::AtHeight(123456),
                };
                let err = execute(
                    deps.as_mut(),
                    mock_env(),
                    info.clone(),
                    ExecuteMsg::Create(create.clone()),
                )
                .unwrap_err();
                assert_eq!(err, ContractError::InvalidId {});
            }

            // Cannot create, no funds
            let info = mock_info(&sender, &[]);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: real_hash(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            let err = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            ).unwrap_err();
            assert_eq!(err, ContractError::EmptyBalance {});

            // Cannot create, expired
            let info = mock_info(&sender, &balance);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: real_hash(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtTime(Timestamp::from_seconds(1)),
            };
            let err = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            ).unwrap_err();
            assert_eq!(err, ContractError::Expired {});

            // Cannot create, invalid hash
            let info = mock_info(&sender, &balance);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: "bu115h17".to_string(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            let err = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            ).unwrap_err();
            assert_eq!(
                err,
                ContractError::ParseError("Invalid character \'u\' at position 1".into())
            );

            // Can create, all valid
            let info = mock_info(&sender, &balance);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: real_hash(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            let res = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            ).unwrap();
            assert_eq!(0, res.messages.len());
            assert_eq!(("action", "create"), res.attributes[0]);

            // Cannot re-create (modify), already existing
            let new_balance = coins(1, "tokens");
            let info = mock_info(&sender, &new_balance);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: real_hash(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            let err = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            ).unwrap_err();
            assert_eq!(err, ContractError::AlreadyExists {});
        }

        /// Test release
        #[test]
        fn test_release() {
            let mut deps = mock_dependencies();

            let info = mock_info("anyone", &[]);
            instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();

            let sender = String::from("sender0001");
            let balance = coins(1000, "tokens");

            let info = mock_info(&sender, &balance);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: real_hash(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Create(create.clone()),
            )
            .unwrap();

            // Anyone can attempt release
            let info = mock_info("somebody", &[]);

            // Cannot release, wrong id
            let release = ExecuteMsg::Release {
                id: "swap0002".to_string(),
                preimage: preimage(),
            };
            let err = execute(deps.as_mut(), mock_env(), info.clone(), release).unwrap_err();
            assert!(matches!(err, ContractError::Std(StdError::NotFound { .. })));

            // Cannot release, invalid hash
            let release = ExecuteMsg::Release {
                id: "swap0001".to_string(),
                preimage: "bu115h17".to_string(),
            };
            let err = execute(deps.as_mut(), mock_env(), info.clone(), release).unwrap_err();
            assert_eq!(
                err,
                ContractError::ParseError("Invalid character \'u\' at position 1".to_string())
            );

            // Cannot release, wrong hash
            let release = ExecuteMsg::Release {
                id: "swap0001".to_string(),
                preimage: hex::encode(b"This is 32 bytes, but incorrect."),
            };
            let err = execute(deps.as_mut(), mock_env(), info, release).unwrap_err();
            assert!(matches!(err, ContractError::InvalidPreimage {}));

            // Cannot release, expired
            let env = mock_env_height(123457);
            let info = mock_info("somebody", &[]);
            let release = ExecuteMsg::Release {
                id: "swap0001".to_string(),
                preimage: preimage(),
            };
            let err = execute(deps.as_mut(), env, info, release).unwrap_err();
            assert!(matches!(err, ContractError::Expired));

            // Can release, valid id, valid hash, and not expired
            let info = mock_info("somebody", &[]);
            let release = ExecuteMsg::Release {
                id: "swap0001".to_string(),
                preimage: preimage(),
            };
            let res = execute(
                deps.as_mut(), mock_env(), info.clone(), release.clone()
            ).unwrap();
            assert_eq!(("action", "release"), res.attributes[0]);
            assert_eq!(1, res.messages.len());
            assert_eq!(
                res.messages[0],
                SubMsg::new(BankMsg::Send {
                    to_address: create.recipient,
                    amount: balance,
                })
            );

            // Cannot release again
            let err = execute(deps.as_mut(), mock_env(), info, release).unwrap_err();
            assert!(matches!(err, ContractError::Std(StdError::NotFound { .. })));
        }

        /// Test refund
        #[test]
        fn test_refund() {
            let mut deps = mock_dependencies();

            let info = mock_info("anyone", &[]);
            instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();

            let sender = String::from("sender0001");
            let balance = coins(1000, "tokens");

            let info = mock_info(&sender, &balance);
            let create = CreateMsg {
                id: "swap0001".to_string(),
                hash: real_hash(),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)).unwrap();

            // Anyone can attempt refund
            let info = mock_info("somebody", &[]);

            // Cannot refund, wrong id
            let refund = ExecuteMsg::Refund {
                id: "swap0002".to_string(),
            };
            let err = execute(deps.as_mut(), mock_env(), info.clone(), refund).unwrap_err();
            assert!(matches!(err, ContractError::Std(StdError::NotFound { .. })));

            // Cannot refund, not expired yet
            let refund = ExecuteMsg::Refund {
                id: "swap0001".to_string(),
            };
            let err = execute(deps.as_mut(), mock_env(), info, refund).unwrap_err();
            assert!(matches!(err, ContractError::NotExpired { .. }));

            // Anyone can refund, if already expired
            let env = mock_env_height(123457);
            let info = mock_info("somebody", &[]);
            let refund = ExecuteMsg::Refund {
                id: "swap0001".to_string(),
            };
            let res = execute(deps.as_mut(), env.clone(), info.clone(), refund.clone()).unwrap();
            assert_eq!(("action", "refund"), res.attributes[0]);
            assert_eq!(1, res.messages.len());
            assert_eq!(
                res.messages[0],
                SubMsg::new(BankMsg::Send {
                    to_address: sender,
                    amount: balance,
                })
            );

            // Cannot refund again
            let err = execute(deps.as_mut(), env, info, refund).unwrap_err();
            assert!(matches!(err, ContractError::Std(StdError::NotFound { .. })));
        }

        /// Test query
        #[test]
        fn test_query() {
            let mut deps = mock_dependencies();

            let info = mock_info("anyone", &[]);
            instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();

            let sender1 = String::from("sender0001");
            let sender2 = String::from("sender0002");
            // Same balance for simplicity
            let balance = coins(1000, "tokens");

            // Create a couple swaps (same hash for simplicity)
            let info = mock_info(&sender1, &balance);
            let create1 = CreateMsg {
                id: "swap0001".to_string(),
                hash: custom_hash(1),
                recipient: "rcpt0001".into(),
                expires: Expiration::AtHeight(123456),
            };
            execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Create(create1.clone()),
            )
            .unwrap();

            let info = mock_info(&sender2, &balance);
            let create2 = CreateMsg {
                id: "swap0002".to_string(),
                hash: custom_hash(2),
                recipient: "rcpt0002".into(),
                expires: Expiration::AtTime(Timestamp::from_seconds(2_000_000_000)),
            };
            execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Create(create2.clone()),
            )
            .unwrap();

            // Get the list of ids
            let query_msg = QueryMsg::List {
                start_after: None,
                limit: None,
            };
            let ids: ListResponse =
                from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
            assert_eq!(2, ids.swaps.len());
            assert_eq!(vec!["swap0001", "swap0002"], ids.swaps);

            // Get the details for the first swap id
            let query_msg = QueryMsg::Details {
                id: ids.swaps[0].clone(),
            };
            let res: DetailsResponse =
                from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
            assert_eq!(
                res,
                DetailsResponse {
                    id: create1.id,
                    hash: create1.hash,
                    recipient: create1.recipient,
                    source: sender1,
                    expires: create1.expires,
                    balance: BalanceHuman::Native(balance.clone()),
                }
            );

            // Get the details for the second swap id
            let query_msg = QueryMsg::Details {
                id: ids.swaps[1].clone(),
            };
            let res: DetailsResponse =
                from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
            assert_eq!(
                res,
                DetailsResponse {
                    id: create2.id,
                    hash: create2.hash,
                    recipient: create2.recipient,
                    source: sender2,
                    expires: create2.expires,
                    balance: BalanceHuman::Native(balance),
                }
            );
        }

        /// test that native and Cw20 swap are successful
        #[test]
        fn test_native_cw20_swap() {
            let mut deps = mock_dependencies();

            // Create the contract
            let info = mock_info("anyone", &[]);
            let res = instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();
            assert_eq!(0, res.messages.len());

            // Native side (offer)
            let native_sender = String::from("a_on_x");
            let native_rcpt = String::from("b_on_x");
            let native_coins = coins(1000, "tokens_native");

            // Create the Native swap offer
            let native_swap_id = "native_swap".to_string();
            let create = CreateMsg {
                id: native_swap_id.clone(),
                hash: real_hash(),
                recipient: native_rcpt.clone(),
                expires: Expiration::AtHeight(123456),
            };
            let info = mock_info(&native_sender, &native_coins);
            let res = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            ).unwrap();
            assert_eq!(0, res.messages.len());
            assert_eq!(("action", "create"), res.attributes[0]);

            // Cw20 side (counter offer (1:1000))
            let cw20_sender = String::from("a_on_y");
            let cw20_rcpt = String::from("b_on_y");
            let cw20_coin = Cw20Coin {
                address: String::from("my_cw20_token"),
                amount: Uint128::new(1),
            };

            // Create the Cw20 side swap counter offer
            let cw20_swap_id = "cw20_swap".to_string();
            let create = CreateMsg {
                id: cw20_swap_id.clone(),
                hash: real_hash(),
                recipient: cw20_rcpt.clone(),
                expires: Expiration::AtHeight(123000),
            };
            let receive = Cw20ReceiveMsg {
                sender: cw20_sender,
                amount: cw20_coin.amount,
                msg: to_binary(&ExecuteMsg::Create(create)).unwrap(),
            };
            let token_contract = cw20_coin.address;
            let info = mock_info(&token_contract, &[]);
            let res: cosmwasm_std::Response = execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(receive)
            )
            .unwrap();
            assert_eq!(0, res.messages.len());
            assert_eq!(("action", "create"), res.attributes[0]);

            // Somebody (typically, A) releases the swap side on the Cw20 (Y) blockchain,
            // using her knowledge of the preimage
            let info = mock_info("somebody", &[]);
            let res = execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Release {
                    id: cw20_swap_id.clone(),
                    preimage: preimage(),
                },
            )
            .unwrap();
            assert_eq!(1, res.messages.len());
            assert_eq!(("action", "release"), res.attributes[0]);
            assert_eq!(("id", cw20_swap_id), res.attributes[1]);

            // Verify the resulting Cw20 transfer message
            let send_msg = Cw20ExecuteMsg::Transfer {
                recipient: cw20_rcpt,
                amount: cw20_coin.amount,
            };
            assert_eq!(
                res.messages[0],
                SubMsg::new(WasmMsg::Execute {
                    contract_addr: token_contract,
                    msg: to_binary(&send_msg).unwrap(),
                    funds: vec![],
                })
            );

            // Now somebody (typically, B) releases the original offer on the Native (X) blockchain,
            // using the (now public) preimage
            let info = mock_info("other_somebody", &[]);

            // First, let's obtain the preimage from the logs of the release() transaction on Y
            let preimage_attr = &res.attributes[2];
            assert_eq!("preimage", preimage_attr.key);
            let preimage = preimage_attr.value.clone();

            let release = ExecuteMsg::Release {
                id: native_swap_id.clone(),
                preimage,
            };
            let res = execute(deps.as_mut(), mock_env(), info, release).unwrap();
            assert_eq!(1, res.messages.len());
            assert_eq!(("action", "release"), res.attributes[0]);
            assert_eq!(("id", native_swap_id), res.attributes[1]);

            // Verify the resulting Native send message
            assert_eq!(
                res.messages[0],
                SubMsg::new(BankMsg::Send {
                    to_address: native_rcpt,
                    amount: native_coins,
                })
            );
        }

        /// test that native swap on same sender and recipient results in failure
        #[test]
        fn test_native_same_sender_recipient() {
            let mut deps = mock_dependencies();

            // Create the contract
            let info = mock_info("anyone", &[]);
            let res = instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();
            assert_eq!(0, res.messages.len());

            // Native side (offer) with same sender and recipient
            let native_sender = String::from("a_on_x");
            let native_rcpt = String::from("a_on_x");
            let native_coins = coins(1000, "tokens_native");

            // Create the Native swap offer
            let native_swap_id = "native_swap".to_string();
            let create = CreateMsg {
                id: native_swap_id.clone(),
                hash: real_hash(),
                recipient: native_rcpt.clone(),
                expires: Expiration::AtHeight(123456),
            };
            let info = mock_info(&native_sender, &native_coins);
            let res = execute(
                deps.as_mut(), mock_env(), info, ExecuteMsg::Create(create)
            );
            // check for certain that the returned result is the correct contract error
            match res {
                Result::Err(ContractError::SameSenderRecipient) => (),
                _ => panic!()
            }
        }
    }


    /// ------------------------ State tests ------------------------ ///

    mod state_test {
        use crate::state::*;
        use cosmwasm_std::testing::MockStorage;
        use cosmwasm_std::{Binary, Addr};

        fn dummy_swap() -> AtomicSwap {
            AtomicSwap {
                recipient: Addr::unchecked("recip"),
                source: Addr::unchecked("source"),
                expires: Default::default(),
                hash: Binary("hash".into()),
                balance: Default::default(),
            }
        }

        #[test]
        fn test_no_swap_ids() {
            let storage = MockStorage::new();
            let ids = all_swap_ids(&storage, None, 10).unwrap();
            assert_eq!(0, ids.len());
        }

        #[test]
        fn test_all_swap_ids() {
            let mut storage = MockStorage::new();
            SWAPS.save(&mut storage, "lazy", &dummy_swap()).unwrap();
            SWAPS.save(&mut storage, "assign", &dummy_swap()).unwrap();
            SWAPS.save(&mut storage, "zen", &dummy_swap()).unwrap();

            let ids = all_swap_ids(&storage, None, 10).unwrap();
            assert_eq!(3, ids.len());
            assert_eq!(
                vec!["assign".to_string(), "lazy".to_string(), "zen".to_string()],
                ids
            )
        }
    }
}