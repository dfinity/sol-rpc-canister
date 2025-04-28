use candid::{CandidType, Deserialize};
use serde::Serialize;

/// Represents errors that can occur during the processing of a Solana transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionError {
    /// An account is already being processed in another transaction in a way
    /// that does not support parallelism
    AccountInUse,
    /// A `Pubkey` appears twice in the transaction's `account_keys`.  Instructions can reference
    /// `Pubkey`s more than once but the message must contain a list with no duplicate keys
    AccountLoadedTwice,
    /// Attempt to debit an account but found no record of a prior credit.
    AccountNotFound,
    /// Attempt to load a program that does not exist
    ProgramAccountNotFound,
    /// The from `Pubkey` does not have sufficient balance to pay the fee to schedule the transaction
    InsufficientFundsForFee,
    /// This account may not be used to pay transaction fees
    InvalidAccountForFee,
    /// The bank has seen this transaction before. This can occur under normal operation
    /// when a UDP packet is duplicated, as a user error from a client not updating
    /// its `recent_blockhash`, or as a double-spend attack.
    AlreadyProcessed,
    /// The bank has not seen the given `recent_blockhash` or the transaction is too old and
    /// the `recent_blockhash` has been discarded.
    BlockhashNotFound,
    /// An error occurred while processing an instruction. The first element of the tuple
    /// indicates the instruction index in which the error occurred.
    InstructionError(u8, InstructionError),
    /// Loader call chain is too deep
    CallChainTooDeep,
    /// Transaction requires a fee but has no signature present
    MissingSignatureForFee,
    /// Transaction contains an invalid account reference
    InvalidAccountIndex,
    /// Transaction did not pass signature verification
    SignatureFailure,
    /// This program may not be used for executing instructions
    InvalidProgramForExecution,
    /// Transaction failed to sanitize accounts offsets correctly
    /// implies that account locks are not taken for this TX, and should
    /// not be unlocked.
    SanitizeFailure,
    /// The transaction was rejected because the cluster is undergoing maintenance.
    ClusterMaintenance,
    /// Transaction processing left an account with an outstanding borrowed reference
    AccountBorrowOutstanding,
    /// Transaction would exceed max Block Cost Limit
    WouldExceedMaxBlockCostLimit,
    /// Transaction version is unsupported
    UnsupportedVersion,
    /// Transaction loads a writable account that cannot be written
    InvalidWritableAccount,
    /// Transaction would exceed max account limit within the block
    WouldExceedMaxAccountCostLimit,
    /// Transaction would exceed account data limit within the block
    WouldExceedAccountDataBlockLimit,
    /// Transaction locked too many accounts
    TooManyAccountLocks,
    /// Address lookup table not found
    AddressLookupTableNotFound,
    /// Attempted to lookup addresses from an account owned by the wrong program
    InvalidAddressLookupTableOwner,
    /// Attempted to lookup addresses from an invalid account
    InvalidAddressLookupTableData,
    /// Address table lookup uses an invalid index
    InvalidAddressLookupTableIndex,
    /// Transaction leaves an account with a lower balance than rent-exempt minimum
    InvalidRentPayingAccount,
    /// Transaction would exceed max Vote Cost Limit
    WouldExceedMaxVoteCostLimit,
    /// Transaction would exceed total account data limit
    WouldExceedAccountDataTotalLimit,
    /// Transaction contains a duplicate instruction that is not allowed
    DuplicateInstruction(u8),
    /// Transaction results in an account with insufficient funds for rent
    InsufficientFundsForRent {
        /// Index of the account.
        account_index: u8,
    },
    /// Transaction exceeded max loaded accounts data size cap
    MaxLoadedAccountsDataSizeExceeded,
    /// LoadedAccountsDataSizeLimit set for transaction must be greater than 0.
    InvalidLoadedAccountsDataSizeLimit,
    /// Sanitized transaction differed before/after feature activiation. Needs to be resanitized.
    ResanitizationNeeded,
    /// Program execution is temporarily restricted on an account.
    ProgramExecutionTemporarilyRestricted {
        /// Index of the account.
        account_index: u8,
    },
    /// The total balance before the transaction does not equal the total balance after the transaction
    UnbalancedTransaction,
    /// Program cache hit max limit.
    ProgramCacheHitMaxLimit,
    /// Commit cancelled internally.
    CommitCancelled,
}

impl From<solana_transaction_error::TransactionError> for TransactionError {
    fn from(error: solana_transaction_error::TransactionError) -> Self {
        use solana_transaction_error::TransactionError;
        match error {
            TransactionError::AccountInUse => Self::AccountInUse,
            TransactionError::AccountLoadedTwice => Self::AccountLoadedTwice,
            TransactionError::AccountNotFound => Self::AccountNotFound,
            TransactionError::ProgramAccountNotFound => Self::ProgramAccountNotFound,
            TransactionError::InsufficientFundsForFee => Self::InsufficientFundsForFee,
            TransactionError::InvalidAccountForFee => Self::InvalidAccountForFee,
            TransactionError::AlreadyProcessed => Self::AlreadyProcessed,
            TransactionError::BlockhashNotFound => Self::BlockhashNotFound,
            TransactionError::InstructionError(index, error) => {
                Self::InstructionError(index, error.into())
            }
            TransactionError::CallChainTooDeep => Self::CallChainTooDeep,
            TransactionError::MissingSignatureForFee => Self::MissingSignatureForFee,
            TransactionError::InvalidAccountIndex => Self::InvalidAccountIndex,
            TransactionError::SignatureFailure => Self::SignatureFailure,
            TransactionError::InvalidProgramForExecution => Self::InvalidProgramForExecution,
            TransactionError::SanitizeFailure => Self::SanitizeFailure,
            TransactionError::ClusterMaintenance => Self::ClusterMaintenance,
            TransactionError::AccountBorrowOutstanding => Self::AccountBorrowOutstanding,
            TransactionError::WouldExceedMaxBlockCostLimit => Self::WouldExceedMaxBlockCostLimit,
            TransactionError::UnsupportedVersion => Self::UnsupportedVersion,
            TransactionError::InvalidWritableAccount => Self::InvalidWritableAccount,
            TransactionError::WouldExceedMaxAccountCostLimit => {
                Self::WouldExceedMaxAccountCostLimit
            }
            TransactionError::WouldExceedAccountDataBlockLimit => {
                Self::WouldExceedAccountDataBlockLimit
            }
            TransactionError::TooManyAccountLocks => Self::TooManyAccountLocks,
            TransactionError::AddressLookupTableNotFound => Self::AddressLookupTableNotFound,
            TransactionError::InvalidAddressLookupTableOwner => {
                Self::InvalidAddressLookupTableOwner
            }
            TransactionError::InvalidAddressLookupTableData => Self::InvalidAddressLookupTableData,
            TransactionError::InvalidAddressLookupTableIndex => {
                Self::InvalidAddressLookupTableIndex
            }
            TransactionError::InvalidRentPayingAccount => Self::InvalidRentPayingAccount,
            TransactionError::WouldExceedMaxVoteCostLimit => Self::WouldExceedMaxVoteCostLimit,
            TransactionError::WouldExceedAccountDataTotalLimit => {
                Self::WouldExceedAccountDataTotalLimit
            }
            TransactionError::DuplicateInstruction(instruction) => {
                Self::DuplicateInstruction(instruction)
            }
            TransactionError::InsufficientFundsForRent { account_index } => {
                Self::InsufficientFundsForRent { account_index }
            }
            TransactionError::MaxLoadedAccountsDataSizeExceeded => {
                Self::MaxLoadedAccountsDataSizeExceeded
            }
            TransactionError::InvalidLoadedAccountsDataSizeLimit => {
                Self::InvalidLoadedAccountsDataSizeLimit
            }
            TransactionError::ResanitizationNeeded => Self::ResanitizationNeeded,
            TransactionError::ProgramExecutionTemporarilyRestricted { account_index } => {
                Self::ProgramExecutionTemporarilyRestricted { account_index }
            }
            TransactionError::UnbalancedTransaction => Self::UnbalancedTransaction,
            TransactionError::ProgramCacheHitMaxLimit => Self::ProgramCacheHitMaxLimit,
            TransactionError::CommitCancelled => Self::CommitCancelled,
        }
    }
}

impl From<TransactionError> for solana_transaction_error::TransactionError {
    fn from(value: TransactionError) -> Self {
        match value {
            TransactionError::AccountInUse => {
                solana_transaction_error::TransactionError::AccountInUse
            }
            TransactionError::AccountLoadedTwice => {
                solana_transaction_error::TransactionError::AccountLoadedTwice
            }
            TransactionError::AccountNotFound => {
                solana_transaction_error::TransactionError::AccountNotFound
            }
            TransactionError::ProgramAccountNotFound => {
                solana_transaction_error::TransactionError::ProgramAccountNotFound
            }
            TransactionError::InsufficientFundsForFee => {
                solana_transaction_error::TransactionError::InsufficientFundsForFee
            }
            TransactionError::InvalidAccountForFee => {
                solana_transaction_error::TransactionError::InvalidAccountForFee
            }
            TransactionError::AlreadyProcessed => {
                solana_transaction_error::TransactionError::AlreadyProcessed
            }
            TransactionError::BlockhashNotFound => {
                solana_transaction_error::TransactionError::BlockhashNotFound
            }
            TransactionError::InstructionError(index, err) => {
                solana_transaction_error::TransactionError::InstructionError(index, err.into())
            }
            TransactionError::CallChainTooDeep => {
                solana_transaction_error::TransactionError::CallChainTooDeep
            }
            TransactionError::MissingSignatureForFee => {
                solana_transaction_error::TransactionError::MissingSignatureForFee
            }
            TransactionError::InvalidAccountIndex => {
                solana_transaction_error::TransactionError::InvalidAccountIndex
            }
            TransactionError::SignatureFailure => {
                solana_transaction_error::TransactionError::SignatureFailure
            }
            TransactionError::InvalidProgramForExecution => {
                solana_transaction_error::TransactionError::InvalidProgramForExecution
            }
            TransactionError::SanitizeFailure => {
                solana_transaction_error::TransactionError::SanitizeFailure
            }
            TransactionError::ClusterMaintenance => {
                solana_transaction_error::TransactionError::ClusterMaintenance
            }
            TransactionError::AccountBorrowOutstanding => {
                solana_transaction_error::TransactionError::AccountBorrowOutstanding
            }
            TransactionError::WouldExceedMaxBlockCostLimit => {
                solana_transaction_error::TransactionError::WouldExceedMaxBlockCostLimit
            }
            TransactionError::UnsupportedVersion => {
                solana_transaction_error::TransactionError::UnsupportedVersion
            }
            TransactionError::InvalidWritableAccount => {
                solana_transaction_error::TransactionError::InvalidWritableAccount
            }
            TransactionError::WouldExceedMaxAccountCostLimit => {
                solana_transaction_error::TransactionError::WouldExceedMaxAccountCostLimit
            }
            TransactionError::WouldExceedAccountDataBlockLimit => {
                solana_transaction_error::TransactionError::WouldExceedAccountDataBlockLimit
            }
            TransactionError::TooManyAccountLocks => {
                solana_transaction_error::TransactionError::TooManyAccountLocks
            }
            TransactionError::AddressLookupTableNotFound => {
                solana_transaction_error::TransactionError::AddressLookupTableNotFound
            }
            TransactionError::InvalidAddressLookupTableOwner => {
                solana_transaction_error::TransactionError::InvalidAddressLookupTableOwner
            }
            TransactionError::InvalidAddressLookupTableData => {
                solana_transaction_error::TransactionError::InvalidAddressLookupTableData
            }
            TransactionError::InvalidAddressLookupTableIndex => {
                solana_transaction_error::TransactionError::InvalidAddressLookupTableIndex
            }
            TransactionError::InvalidRentPayingAccount => {
                solana_transaction_error::TransactionError::InvalidRentPayingAccount
            }
            TransactionError::WouldExceedMaxVoteCostLimit => {
                solana_transaction_error::TransactionError::WouldExceedMaxVoteCostLimit
            }
            TransactionError::WouldExceedAccountDataTotalLimit => {
                solana_transaction_error::TransactionError::WouldExceedAccountDataTotalLimit
            }
            TransactionError::DuplicateInstruction(instruction) => {
                solana_transaction_error::TransactionError::DuplicateInstruction(instruction)
            }
            TransactionError::InsufficientFundsForRent { account_index } => {
                solana_transaction_error::TransactionError::InsufficientFundsForRent {
                    account_index,
                }
            }
            TransactionError::MaxLoadedAccountsDataSizeExceeded => {
                solana_transaction_error::TransactionError::MaxLoadedAccountsDataSizeExceeded
            }
            TransactionError::InvalidLoadedAccountsDataSizeLimit => {
                solana_transaction_error::TransactionError::InvalidLoadedAccountsDataSizeLimit
            }
            TransactionError::ResanitizationNeeded => {
                solana_transaction_error::TransactionError::ResanitizationNeeded
            }
            TransactionError::ProgramExecutionTemporarilyRestricted { account_index } => {
                solana_transaction_error::TransactionError::ProgramExecutionTemporarilyRestricted {
                    account_index,
                }
            }
            TransactionError::UnbalancedTransaction => {
                solana_transaction_error::TransactionError::UnbalancedTransaction
            }
            TransactionError::ProgramCacheHitMaxLimit => {
                solana_transaction_error::TransactionError::ProgramCacheHitMaxLimit
            }
            TransactionError::CommitCancelled => {
                solana_transaction_error::TransactionError::CommitCancelled
            }
        }
    }
}

/// Represents errors that can occur during the execution of a specific instruction within a Solana
/// transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum InstructionError {
    /// Deprecated! Use CustomError instead!
    /// The program instruction returned an error
    GenericError,
    /// The arguments provided to a program were invalid
    InvalidArgument,
    /// An instruction's data contents were invalid
    InvalidInstructionData,
    /// An account's data contents was invalid
    InvalidAccountData,
    /// An account's data was too small
    AccountDataTooSmall,
    /// An account's balance was too small to complete the instruction
    InsufficientFunds,
    /// The account did not have the expected program id
    IncorrectProgramId,
    /// A signature was required but not found
    MissingRequiredSignature,
    /// An initialize instruction was sent to an account that has already been initialized.
    AccountAlreadyInitialized,
    /// An attempt to operate on an account that hasn't been initialized.
    UninitializedAccount,
    /// Program's instruction lamport balance does not equal the balance after the instruction
    UnbalancedInstruction,
    /// Program illegally modified an account's program id
    ModifiedProgramId,
    /// Program spent the lamports of an account that doesn't belong to it
    ExternalAccountLamportSpend,
    /// Program modified the data of an account that doesn't belong to it
    ExternalAccountDataModified,
    /// Read-only account's lamports modified
    ReadonlyLamportChange,
    /// Read-only account's data was modified
    ReadonlyDataModified,
    /// An account was referenced more than once in a single instruction
    // Deprecated, instructions can now contain duplicate accounts
    DuplicateAccountIndex,
    /// Executable bit on account changed, but shouldn't have
    ExecutableModified,
    /// Rent_epoch account changed, but shouldn't have
    RentEpochModified,
    /// The instruction expected additional account keys
    NotEnoughAccountKeys,
    /// Program other than the account's owner changed the size of the account data
    AccountDataSizeChanged,
    /// The instruction expected an executable account
    AccountNotExecutable,
    /// Failed to borrow a reference to account data, already borrowed
    AccountBorrowFailed,
    /// Account data has an outstanding reference after a program's execution
    AccountBorrowOutstanding,
    /// The same account was multiply passed to an on-chain program's entrypoint, but the program
    /// modified them differently.  A program can only modify one instance of the account because
    /// the runtime cannot determine which changes to pick or how to merge them if both are modified
    DuplicateAccountOutOfSync,
    /// Allows on-chain programs to implement program-specific error types and see them returned
    /// by the Solana runtime. A program-specific error may be any type that is represented as
    /// or serialized to a u32 integer.
    Custom(u32),
    /// The return value from the program was invalid.  Valid errors are either a defined builtin
    /// error value or a user-defined error in the lower 32 bits.
    InvalidError,
    /// Executable account's data was modified
    ExecutableDataModified,
    /// Executable account's lamports modified
    ExecutableLamportChange,
    /// Executable accounts must be rent exempt
    ExecutableAccountNotRentExempt,
    /// Unsupported program id
    UnsupportedProgramId,
    /// Cross-program invocation call depth too deep
    CallDepth,
    /// An account required by the instruction is missing
    MissingAccount,
    /// Cross-program invocation reentrancy not allowed for this instruction
    ReentrancyNotAllowed,
    /// Length of the seed is too long for address generation
    MaxSeedLengthExceeded,
    /// Provided seeds do not result in a valid address
    InvalidSeeds,
    /// Failed to reallocate account data of this length
    InvalidRealloc,
    /// Computational budget exceeded
    ComputationalBudgetExceeded,
    /// Cross-program invocation with unauthorized signer or writable account
    PrivilegeEscalation,
    /// Failed to create program execution environment
    ProgramEnvironmentSetupFailure,
    /// Program failed to complete
    ProgramFailedToComplete,
    /// Program failed to compile
    ProgramFailedToCompile,
    /// Account is immutable
    Immutable,
    /// Incorrect authority provided
    IncorrectAuthority,
    /// Failed to serialize or deserialize account data
    ///
    /// Warning: This error should never be emitted by the runtime.
    ///
    /// This error includes strings from the underlying 3rd party Borsh crate
    /// which can be dangerous because the error strings could change across
    /// Borsh versions. Only programs can use this error because they are
    /// consistent across Solana software versions.
    ///
    BorshIoError(String),
    /// An account does not have enough lamports to be rent-exempt
    AccountNotRentExempt,
    /// Invalid account owner
    InvalidAccountOwner,
    /// Program arithmetic overflowed
    ArithmeticOverflow,
    /// Unsupported sysvar
    UnsupportedSysvar,
    /// Illegal account owner
    IllegalOwner,
    /// Accounts data allocations exceeded the maximum allowed per transaction
    MaxAccountsDataAllocationsExceeded,
    /// Max accounts exceeded
    MaxAccountsExceeded,
    /// Max instruction trace length exceeded
    MaxInstructionTraceLengthExceeded,
    /// Builtin programs must consume compute units
    BuiltinProgramsMustConsumeComputeUnits,
    // Note: For any new error added here an equivalent ProgramError and its
    // conversions must also be added
}

impl From<solana_instruction::error::InstructionError> for InstructionError {
    fn from(value: solana_instruction::error::InstructionError) -> Self {
        use solana_instruction::error::InstructionError;
        match value {
            InstructionError::GenericError => Self::GenericError,
            InstructionError::InvalidArgument => Self::InvalidArgument,
            InstructionError::InvalidInstructionData => Self::InvalidInstructionData,
            InstructionError::InvalidAccountData => Self::InvalidAccountData,
            InstructionError::AccountDataTooSmall => Self::AccountDataTooSmall,
            InstructionError::InsufficientFunds => Self::InsufficientFunds,
            InstructionError::IncorrectProgramId => Self::IncorrectProgramId,
            InstructionError::MissingRequiredSignature => Self::MissingRequiredSignature,
            InstructionError::AccountAlreadyInitialized => Self::AccountAlreadyInitialized,
            InstructionError::UninitializedAccount => Self::UninitializedAccount,
            InstructionError::UnbalancedInstruction => Self::UnbalancedInstruction,
            InstructionError::ModifiedProgramId => Self::ModifiedProgramId,
            InstructionError::ExternalAccountLamportSpend => Self::ExternalAccountLamportSpend,
            InstructionError::ExternalAccountDataModified => Self::ExternalAccountDataModified,
            InstructionError::ReadonlyLamportChange => Self::ReadonlyLamportChange,
            InstructionError::ReadonlyDataModified => Self::ReadonlyDataModified,
            InstructionError::DuplicateAccountIndex => Self::DuplicateAccountIndex,
            InstructionError::ExecutableModified => Self::ExecutableModified,
            InstructionError::RentEpochModified => Self::RentEpochModified,
            InstructionError::NotEnoughAccountKeys => Self::NotEnoughAccountKeys,
            InstructionError::AccountDataSizeChanged => Self::AccountDataSizeChanged,
            InstructionError::AccountNotExecutable => Self::AccountNotExecutable,
            InstructionError::AccountBorrowFailed => Self::AccountBorrowFailed,
            InstructionError::AccountBorrowOutstanding => Self::AccountBorrowOutstanding,
            InstructionError::DuplicateAccountOutOfSync => Self::DuplicateAccountOutOfSync,
            InstructionError::Custom(err) => Self::Custom(err),
            InstructionError::InvalidError => Self::InvalidError,
            InstructionError::ExecutableDataModified => Self::ExecutableDataModified,
            InstructionError::ExecutableLamportChange => Self::ExecutableLamportChange,
            InstructionError::ExecutableAccountNotRentExempt => {
                Self::ExecutableAccountNotRentExempt
            }
            InstructionError::UnsupportedProgramId => Self::UnsupportedProgramId,
            InstructionError::CallDepth => Self::CallDepth,
            InstructionError::MissingAccount => Self::MissingAccount,
            InstructionError::ReentrancyNotAllowed => Self::ReentrancyNotAllowed,
            InstructionError::MaxSeedLengthExceeded => Self::MaxSeedLengthExceeded,
            InstructionError::InvalidSeeds => Self::InvalidSeeds,
            InstructionError::InvalidRealloc => Self::InvalidRealloc,
            InstructionError::ComputationalBudgetExceeded => Self::ComputationalBudgetExceeded,
            InstructionError::PrivilegeEscalation => Self::PrivilegeEscalation,
            InstructionError::ProgramEnvironmentSetupFailure => {
                Self::ProgramEnvironmentSetupFailure
            }
            InstructionError::ProgramFailedToComplete => Self::ProgramFailedToComplete,
            InstructionError::ProgramFailedToCompile => Self::ProgramFailedToCompile,
            InstructionError::Immutable => Self::Immutable,
            InstructionError::IncorrectAuthority => Self::IncorrectAuthority,
            InstructionError::BorshIoError(err) => Self::BorshIoError(err),
            InstructionError::AccountNotRentExempt => Self::AccountNotRentExempt,
            InstructionError::InvalidAccountOwner => Self::InvalidAccountOwner,
            InstructionError::ArithmeticOverflow => Self::ArithmeticOverflow,
            InstructionError::UnsupportedSysvar => Self::UnsupportedSysvar,
            InstructionError::IllegalOwner => Self::IllegalOwner,
            InstructionError::MaxAccountsDataAllocationsExceeded => {
                Self::MaxAccountsDataAllocationsExceeded
            }
            InstructionError::MaxAccountsExceeded => Self::MaxAccountsExceeded,
            InstructionError::MaxInstructionTraceLengthExceeded => {
                Self::MaxInstructionTraceLengthExceeded
            }
            InstructionError::BuiltinProgramsMustConsumeComputeUnits => {
                Self::BuiltinProgramsMustConsumeComputeUnits
            }
        }
    }
}

impl From<InstructionError> for solana_instruction::error::InstructionError {
    fn from(value: InstructionError) -> Self {
        match value {
            InstructionError::GenericError => {
                solana_instruction::error::InstructionError::GenericError
            }
            InstructionError::InvalidArgument => {
                solana_instruction::error::InstructionError::InvalidArgument
            }
            InstructionError::InvalidInstructionData => {
                solana_instruction::error::InstructionError::InvalidInstructionData
            }
            InstructionError::InvalidAccountData => {
                solana_instruction::error::InstructionError::InvalidAccountData
            }
            InstructionError::AccountDataTooSmall => {
                solana_instruction::error::InstructionError::AccountDataTooSmall
            }
            InstructionError::InsufficientFunds => {
                solana_instruction::error::InstructionError::InsufficientFunds
            }
            InstructionError::IncorrectProgramId => {
                solana_instruction::error::InstructionError::IncorrectProgramId
            }
            InstructionError::MissingRequiredSignature => {
                solana_instruction::error::InstructionError::MissingRequiredSignature
            }
            InstructionError::AccountAlreadyInitialized => {
                solana_instruction::error::InstructionError::AccountAlreadyInitialized
            }
            InstructionError::UninitializedAccount => {
                solana_instruction::error::InstructionError::UninitializedAccount
            }
            InstructionError::UnbalancedInstruction => {
                solana_instruction::error::InstructionError::UnbalancedInstruction
            }
            InstructionError::ModifiedProgramId => {
                solana_instruction::error::InstructionError::ModifiedProgramId
            }
            InstructionError::ExternalAccountLamportSpend => {
                solana_instruction::error::InstructionError::ExternalAccountLamportSpend
            }
            InstructionError::ExternalAccountDataModified => {
                solana_instruction::error::InstructionError::ExternalAccountDataModified
            }
            InstructionError::ReadonlyLamportChange => {
                solana_instruction::error::InstructionError::ReadonlyLamportChange
            }
            InstructionError::ReadonlyDataModified => {
                solana_instruction::error::InstructionError::ReadonlyDataModified
            }
            InstructionError::DuplicateAccountIndex => {
                solana_instruction::error::InstructionError::DuplicateAccountIndex
            }
            InstructionError::ExecutableModified => {
                solana_instruction::error::InstructionError::ExecutableModified
            }
            InstructionError::RentEpochModified => {
                solana_instruction::error::InstructionError::RentEpochModified
            }
            InstructionError::NotEnoughAccountKeys => {
                solana_instruction::error::InstructionError::NotEnoughAccountKeys
            }
            InstructionError::AccountDataSizeChanged => {
                solana_instruction::error::InstructionError::AccountDataSizeChanged
            }
            InstructionError::AccountNotExecutable => {
                solana_instruction::error::InstructionError::AccountNotExecutable
            }
            InstructionError::AccountBorrowFailed => {
                solana_instruction::error::InstructionError::AccountBorrowFailed
            }
            InstructionError::AccountBorrowOutstanding => {
                solana_instruction::error::InstructionError::AccountBorrowOutstanding
            }
            InstructionError::DuplicateAccountOutOfSync => {
                solana_instruction::error::InstructionError::DuplicateAccountOutOfSync
            }
            InstructionError::Custom(err) => {
                solana_instruction::error::InstructionError::Custom(err)
            }
            InstructionError::InvalidError => {
                solana_instruction::error::InstructionError::InvalidError
            }
            InstructionError::ExecutableDataModified => {
                solana_instruction::error::InstructionError::ExecutableDataModified
            }
            InstructionError::ExecutableLamportChange => {
                solana_instruction::error::InstructionError::ExecutableLamportChange
            }
            InstructionError::ExecutableAccountNotRentExempt => {
                solana_instruction::error::InstructionError::ExecutableAccountNotRentExempt
            }
            InstructionError::UnsupportedProgramId => {
                solana_instruction::error::InstructionError::UnsupportedProgramId
            }
            InstructionError::CallDepth => solana_instruction::error::InstructionError::CallDepth,
            InstructionError::MissingAccount => {
                solana_instruction::error::InstructionError::MissingAccount
            }
            InstructionError::ReentrancyNotAllowed => {
                solana_instruction::error::InstructionError::ReentrancyNotAllowed
            }
            InstructionError::MaxSeedLengthExceeded => {
                solana_instruction::error::InstructionError::MaxSeedLengthExceeded
            }
            InstructionError::InvalidSeeds => {
                solana_instruction::error::InstructionError::InvalidSeeds
            }
            InstructionError::InvalidRealloc => {
                solana_instruction::error::InstructionError::InvalidRealloc
            }
            InstructionError::ComputationalBudgetExceeded => {
                solana_instruction::error::InstructionError::ComputationalBudgetExceeded
            }
            InstructionError::PrivilegeEscalation => {
                solana_instruction::error::InstructionError::PrivilegeEscalation
            }
            InstructionError::ProgramEnvironmentSetupFailure => {
                solana_instruction::error::InstructionError::ProgramEnvironmentSetupFailure
            }
            InstructionError::ProgramFailedToComplete => {
                solana_instruction::error::InstructionError::ProgramFailedToComplete
            }
            InstructionError::ProgramFailedToCompile => {
                solana_instruction::error::InstructionError::ProgramFailedToCompile
            }
            InstructionError::Immutable => solana_instruction::error::InstructionError::Immutable,
            InstructionError::IncorrectAuthority => {
                solana_instruction::error::InstructionError::IncorrectAuthority
            }
            InstructionError::BorshIoError(err) => {
                solana_instruction::error::InstructionError::BorshIoError(err)
            }
            InstructionError::AccountNotRentExempt => {
                solana_instruction::error::InstructionError::AccountNotRentExempt
            }
            InstructionError::InvalidAccountOwner => {
                solana_instruction::error::InstructionError::InvalidAccountOwner
            }
            InstructionError::ArithmeticOverflow => {
                solana_instruction::error::InstructionError::ArithmeticOverflow
            }
            InstructionError::UnsupportedSysvar => {
                solana_instruction::error::InstructionError::UnsupportedSysvar
            }
            InstructionError::IllegalOwner => {
                solana_instruction::error::InstructionError::IllegalOwner
            }
            InstructionError::MaxAccountsDataAllocationsExceeded => {
                solana_instruction::error::InstructionError::MaxAccountsDataAllocationsExceeded
            }
            InstructionError::MaxAccountsExceeded => {
                solana_instruction::error::InstructionError::MaxAccountsExceeded
            }
            InstructionError::MaxInstructionTraceLengthExceeded => {
                solana_instruction::error::InstructionError::MaxInstructionTraceLengthExceeded
            }
            InstructionError::BuiltinProgramsMustConsumeComputeUnits => {
                solana_instruction::error::InstructionError::BuiltinProgramsMustConsumeComputeUnits
            }
        }
    }
}
