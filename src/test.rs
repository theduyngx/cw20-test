/// Test module - we haven't implemented any logic here other than the Cw20 base so it might not matter a lot
/// at the moment. But this can be extended however required.
#[cfg(test)]
mod test {
    use crate::contract::*;
    use cosmwasm_std::{Uint128, MessageInfo, Env, Response};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw20::{Cw20Coin, TokenInfoResponse};
    use cw20_base::contract::query_token_info;
    use cw20_base::msg::InstantiateMsg;

    /// basic instantiation test verifying instantiate response and the token's information
    /// by querying.
    #[test]
    fn instantiate_test() {
        // instantiate and assert response
        let mut deps = mock_dependencies();
        let env : Env            = mock_env();
        let info: MessageInfo    = mock_info(&"sender", &[]);
        let msg : InstantiateMsg = InstantiateMsg {
            name             : "GOLD".to_string(),
            symbol           : "GLD".to_string(),
            decimals         : 10,
            initial_balances : vec![
                Cw20Coin {
                    address  : String::from("sender"),
                    amount   : Uint128::new(1928334),
                }
            ],
            mint             : None,
            marketing        : None,
        };
        let res: Response = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // assert queried token matches the instantiated token
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap(),
            TokenInfoResponse {
                name: "GOLD".to_string(),
                symbol: "GLD".to_string(),
                decimals: 10,
                total_supply: Uint128::new(1928334),
            }
        );
    }
}
