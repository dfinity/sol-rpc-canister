pub type ProviderId = &'static str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub provider_id: ProviderId,
    pub cluster: sol_rpc_types::SolanaCluster,
    pub access: RpcAccess,
    pub alias: Option<sol_rpc_types::RpcService>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcAccess {
    Authenticated {
        auth: RpcAuth,
        /// Public URL to use when the API key is not available.
        public_url: Option<&'static str>,
    },
    Unauthenticated {
        public_url: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcAuth {
    /// API key will be used in an Authorization header as Bearer token, e.g.,
    /// `Authorization: Bearer API_KEY`
    BearerToken {
        url: &'static str,
    },
    UrlParameter {
        url_pattern: &'static str,
    },
}
