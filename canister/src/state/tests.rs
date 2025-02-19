use crate::state::{init_state, mutate_state, read_state, State};
use candid::Principal;

#[test]
fn test_api_key_principals() {
    init_state(State::default());

    let principal1 =
        Principal::from_text("k5dlc-ijshq-lsyre-qvvpq-2bnxr-pb26c-ag3sc-t6zo5-rdavy-recje-zqe")
            .unwrap();
    let principal2 =
        Principal::from_text("yxhtl-jlpgx-wqnzc-ysego-h6yqe-3zwfo-o3grn-gvuhm-nz3kv-ainub-6ae")
            .unwrap();
    assert!(!is_api_key_principal(&principal1));
    assert!(!is_api_key_principal(&principal2));

    set_api_key_principals(vec![principal1]);
    assert!(is_api_key_principal(&principal1));
    assert!(!is_api_key_principal(&principal2));

    set_api_key_principals(vec![principal2]);
    assert!(!is_api_key_principal(&principal1));
    assert!(is_api_key_principal(&principal2));

    set_api_key_principals(vec![principal1, principal2]);
    assert!(is_api_key_principal(&principal1));
    assert!(is_api_key_principal(&principal2));

    set_api_key_principals(vec![]);
    assert!(!is_api_key_principal(&principal1));
    assert!(!is_api_key_principal(&principal2));
}

fn set_api_key_principals(new_principals: Vec<Principal>) {
    mutate_state(|state| state.set_api_key_principals(new_principals));
}

fn is_api_key_principal(principal: &Principal) -> bool {
    read_state(|state| state.is_api_key_principal(principal))
}
