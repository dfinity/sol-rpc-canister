mod sol_rpc;
#[cfg(test)]
mod tests;

use crate::{
    logs::Priority,
    providers::Providers,
    rpc_client::sol_rpc::{ResponseSizeEstimate, ResponseTransform, HEADER_SIZE_LIMIT},
};
use canhttp::http::json::JsonRpcRequest;
use canlog::log;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{
    ConsensusStrategy, GetSlotParams, ProviderError, RpcConfig, RpcError, RpcResult, RpcSource,
    RpcSources,
};
use solana_clock::Slot;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

pub async fn call<I, O>(
    provider: &RpcSource,
    request: JsonRpcRequest<I>,
    max_response_size: u64,
) -> Result<O, RpcError>
where
    I: Serialize + Clone + Debug,
    O: Debug + DeserializeOwned,
{
    sol_rpc::call::<_, _>(
        false,
        provider,
        request,
        ResponseSizeEstimate::new(max_response_size),
        &Some(ResponseTransform::Raw),
    )
    .await
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolRpcClient {
    providers: Providers,
    config: RpcConfig,
}

impl SolRpcClient {
    pub fn new(source: RpcSources, config: Option<RpcConfig>) -> Result<Self, ProviderError> {
        let config = config.unwrap_or_default();
        let strategy = config.response_consensus.clone().unwrap_or_default();
        Ok(Self {
            providers: Providers::new(source, strategy)?,
            config,
        })
    }

    fn providers(&self) -> &BTreeSet<RpcSource> {
        &self.providers.sources
    }

    fn response_size_estimate(&self, estimate: u64) -> ResponseSizeEstimate {
        ResponseSizeEstimate::new(self.config.response_size_estimate.unwrap_or(estimate))
    }

    fn consensus_strategy(&self) -> ConsensusStrategy {
        self.config
            .response_consensus
            .as_ref()
            .cloned()
            .unwrap_or_default()
    }

    /// Query all providers in parallel and return all results.
    /// It's up to the caller to decide how to handle the results, which could be inconsistent
    /// (e.g., if different providers gave different responses).
    /// This method is useful for querying data that is critical for the system to ensure that
    /// there is no single point of failure.
    async fn parallel_call<I, O>(
        &self,
        method: impl Into<String>,
        params: I,
        response_size_estimate: ResponseSizeEstimate,
        response_transform: &Option<ResponseTransform>,
    ) -> MultiCallResults<O>
    where
        I: Serialize + Clone + Debug,
        O: Debug + DeserializeOwned,
    {
        let providers = self.providers();
        let request = JsonRpcRequest::new(method, params);
        let results = {
            let mut fut = Vec::with_capacity(providers.len());
            for provider in providers {
                log!(
                    Priority::Debug,
                    "[parallel_call]: will call provider: {:?}",
                    provider
                );
                fut.push(async {
                    sol_rpc::call::<_, _>(
                        true,
                        provider,
                        request.clone(),
                        response_size_estimate,
                        response_transform,
                    )
                    .await
                });
            }
            futures::future::join_all(fut).await
        };
        MultiCallResults::from_non_empty_iter(providers.iter().cloned().zip(results.into_iter()))
    }

    /// Query the Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method.
    pub async fn get_slot(&self, params: GetSlotParams) -> Result<Slot, MultiCallError<Slot>> {
        self.parallel_call(
            "getSlot",
            vec![params],
            self.response_size_estimate(1024 + HEADER_SIZE_LIMIT),
            &Some(ResponseTransform::GetSlot),
        )
        .await
        .reduce(self.consensus_strategy())
    }
}

/// Aggregates responses of different providers to the same query.
/// Guaranteed to be non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiCallResults<T> {
    ok_results: BTreeMap<RpcSource, T>,
    errors: BTreeMap<RpcSource, RpcError>,
}

impl<T> Default for MultiCallResults<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MultiCallResults<T> {
    pub fn new() -> Self {
        Self {
            ok_results: BTreeMap::new(),
            errors: BTreeMap::new(),
        }
    }

    pub fn from_non_empty_iter<I: IntoIterator<Item = (RpcSource, RpcResult<T>)>>(iter: I) -> Self {
        let mut results = Self::new();
        for (provider, result) in iter {
            results.insert_once(provider, result);
        }
        if results.is_empty() {
            panic!("BUG: MultiCallResults cannot be empty!")
        }
        results
    }

    fn is_empty(&self) -> bool {
        self.ok_results.is_empty() && self.errors.is_empty()
    }

    fn insert_once(&mut self, provider: RpcSource, result: RpcResult<T>) {
        match result {
            Ok(value) => {
                assert!(!self.errors.contains_key(&provider));
                assert!(self.ok_results.insert(provider, value).is_none());
            }
            Err(error) => {
                assert!(!self.ok_results.contains_key(&provider));
                assert!(self.errors.insert(provider, error).is_none());
            }
        }
    }

    pub fn into_vec(self) -> Vec<(RpcSource, RpcResult<T>)> {
        self.ok_results
            .into_iter()
            .map(|(provider, result)| (provider, Ok(result)))
            .chain(
                self.errors
                    .into_iter()
                    .map(|(provider, error)| (provider, Err(error))),
            )
            .collect()
    }

    fn group_errors(&self) -> BTreeMap<&RpcError, BTreeSet<&RpcSource>> {
        let mut errors: BTreeMap<_, _> = BTreeMap::new();
        for (provider, error) in self.errors.iter() {
            errors
                .entry(error)
                .or_insert_with(BTreeSet::new)
                .insert(provider);
        }
        errors
    }
}

impl<T: PartialEq> MultiCallResults<T> {
    /// Expects all results to be ok or return the following error:
    /// * MultiCallError::ConsistentError: all errors are the same and there is no ok results.
    /// * MultiCallError::InconsistentResults: in all other cases.
    fn all_ok(self) -> Result<BTreeMap<RpcSource, T>, MultiCallError<T>> {
        if self.errors.is_empty() {
            return Ok(self.ok_results);
        }
        Err(self.expect_error())
    }

    fn expect_error(self) -> MultiCallError<T> {
        let errors = self.group_errors();
        match errors.len() {
            0 => {
                panic!("BUG: errors should be non-empty")
            }
            1 if self.ok_results.is_empty() => {
                MultiCallError::ConsistentError(errors.into_keys().next().unwrap().clone())
            }
            _ => MultiCallError::InconsistentResults(self),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MultiCallError<T> {
    ConsistentError(RpcError),
    InconsistentResults(MultiCallResults<T>),
}

impl<T: Debug + PartialEq + Clone + Serialize> MultiCallResults<T> {
    pub fn reduce(self, strategy: ConsensusStrategy) -> Result<T, MultiCallError<T>> {
        match strategy {
            ConsensusStrategy::Equality => self.reduce_with_equality(),
            ConsensusStrategy::Threshold { total: _, min } => self.reduce_with_threshold(min),
        }
    }

    fn reduce_with_equality(self) -> Result<T, MultiCallError<T>> {
        let mut results = self.all_ok()?.into_iter();
        let (base_node_provider, base_result) = results
            .next()
            .expect("BUG: MultiCallResults is guaranteed to be non-empty");
        let mut inconsistent_results: Vec<_> = results
            .filter(|(_provider, result)| result != &base_result)
            .collect();
        if !inconsistent_results.is_empty() {
            inconsistent_results.push((base_node_provider, base_result));
            let error = MultiCallError::InconsistentResults(MultiCallResults::from_non_empty_iter(
                inconsistent_results
                    .into_iter()
                    .map(|(provider, result)| (provider, Ok(result))),
            ));
            log!(
                Priority::Info,
                "[reduce_with_equality]: inconsistent results {error:?}"
            );
            return Err(error);
        }
        Ok(base_result)
    }

    fn reduce_with_threshold(self, min: u8) -> Result<T, MultiCallError<T>> {
        assert!(min > 0, "BUG: min must be greater than 0");
        if self.ok_results.len() < min as usize {
            // At least total >= min were queried,
            // so there is at least one error
            return Err(self.expect_error());
        }
        let distribution = ResponseDistribution::from_non_empty_iter(self.ok_results.clone());
        let (most_likely_response, providers) = distribution
            .most_frequent()
            .expect("BUG: distribution should be non-empty");
        if providers.len() >= min as usize {
            Ok(most_likely_response.clone())
        } else {
            log!(
                Priority::Info,
                "[reduce_with_threshold]: too many inconsistent ok responses to reach threshold of {min}, results: {self:?}"
            );
            Err(MultiCallError::InconsistentResults(self))
        }
    }
}

/// Distribution of responses observed from different providers.
///
/// From the API point of view, it emulates a map from a response instance to a set of providers that returned it.
/// At the implementation level, to avoid requiring `T` to have a total order (i.e., must implements `Ord` if it were to be used as keys in a `BTreeMap`) which might not always be meaningful,
/// we use as key the hash of the serialized response instance.
struct ResponseDistribution<T> {
    hashes: BTreeMap<[u8; 32], T>,
    responses: BTreeMap<[u8; 32], BTreeSet<RpcSource>>,
}

impl<T> Default for ResponseDistribution<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ResponseDistribution<T> {
    pub fn new() -> Self {
        Self {
            hashes: BTreeMap::new(),
            responses: BTreeMap::new(),
        }
    }

    /// Returns the most frequent response and the set of providers that returned it.
    pub fn most_frequent(&self) -> Option<(&T, &BTreeSet<RpcSource>)> {
        self.responses
            .iter()
            .max_by_key(|(_hash, providers)| providers.len())
            .map(|(hash, providers)| {
                (
                    self.hashes.get(hash).expect("BUG: hash should be present"),
                    providers,
                )
            })
    }
}

impl<T: Debug + PartialEq + Serialize> ResponseDistribution<T> {
    pub fn from_non_empty_iter<I: IntoIterator<Item = (RpcSource, T)>>(iter: I) -> Self {
        let mut distribution = Self::new();
        for (provider, result) in iter {
            distribution.insert_once(provider, result);
        }
        distribution
    }

    pub fn insert_once(&mut self, provider: RpcSource, result: T) {
        use ic_sha3::Keccak256;
        let hash = Keccak256::hash(serde_json::to_vec(&result).expect("BUG: failed to serialize"));
        match self.hashes.get(&hash) {
            Some(existing_result) => {
                assert_eq!(
                    existing_result, &result,
                    "BUG: different results once serialized have the same hash"
                );
                let providers = self
                    .responses
                    .get_mut(&hash)
                    .expect("BUG: hash is guaranteed to be present");
                assert!(
                    providers.insert(provider),
                    "BUG: provider is already present"
                );
            }
            None => {
                assert_eq!(self.hashes.insert(hash, result), None);
                let providers = BTreeSet::from_iter(std::iter::once(provider));
                assert_eq!(self.responses.insert(hash, providers), None);
            }
        }
    }
}
