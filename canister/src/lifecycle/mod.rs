use crate::state::{init_state, mutate_state, State};
use sol_rpc_types::InstallArgs;

pub fn init(args: InstallArgs) {
    // TODO XC-286: Add logging
    // log!(
    //     INFO,
    //     "[init]: initialized SOL RPC canister with arg: {:?}",
    //     args
    // );
    init_state(State::from(args));
}

pub fn post_upgrade(args: Option<InstallArgs>) {
    // TODO XC-286: Add logging
    // log!(
    //     INFO,
    //     "[init]: upgraded SOL RPC canister with arg: {:?}",
    //     args
    // );
    pub fn update_state(args: InstallArgs) {
        if let Some(api_key_principals) = args.manage_api_keys {
            mutate_state(|s| s.set_api_key_principals(api_key_principals));
        }
    }
}
