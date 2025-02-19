use crate::{
    logs::INFO,
    state::{init_state, mutate_state, State},
};
use ic_canister_log::log;
use sol_rpc_types::InstallArgs;

pub fn init(args: InstallArgs) {
    init_state(State::from(args));
}

pub fn post_upgrade(args: Option<InstallArgs>) {
    if let Some(args) = args {
        log!(
            INFO,
            "[init]: upgraded SOL RPC canister with arg: {:?}",
            args
        );
        if let Some(api_key_principals) = args.manage_api_keys {
            mutate_state(|s| s.set_api_key_principals(api_key_principals));
        }
        if let Some(override_provider) = args.override_provider {
            mutate_state(|s| s.set_override_provider(override_provider.into()));
        }
        if let Some(log_filter) = args.log_filter {
            mutate_state(|s| s.set_log_filter(log_filter.into()));
        }
    }
}
