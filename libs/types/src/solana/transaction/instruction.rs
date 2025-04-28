use crate::RpcError;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use solana_transaction_status_client_types::{
    UiCompiledInstruction, UiInnerInstructions, UiInstruction,
};

/// List of [inner instructions](https://solana.com/de/docs/rpc/json-structures#inner-instructions)
/// for a Solana transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct InnerInstructions {
    ///  Index of the transaction instruction from which the inner instruction(s) originated.
    pub index: u8,
    /// Ordered list of inner program instructions that were invoked during a single transaction
    /// instruction.
    pub instructions: Vec<Instruction>,
}

impl TryFrom<UiInnerInstructions> for InnerInstructions {
    type Error = RpcError;

    fn try_from(instructions: UiInnerInstructions) -> Result<Self, Self::Error> {
        Ok(Self {
            index: instructions.index,
            instructions: instructions
                .instructions
                .into_iter()
                .map(Instruction::try_from)
                .collect::<Result<Vec<Instruction>, Self::Error>>()?,
        })
    }
}

impl From<InnerInstructions> for UiInnerInstructions {
    fn from(instructions: InnerInstructions) -> Self {
        Self {
            index: instructions.index,
            instructions: instructions
                .instructions
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// A directive for a single invocation of a Solana program.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum Instruction {
    /// A compiled Solana instruction.
    #[serde(rename = "compiled")]
    Compiled(CompiledInstruction),
}

impl From<Instruction> for UiInstruction {
    fn from(instruction: Instruction) -> Self {
        match instruction {
            Instruction::Compiled(compiled) => Self::Compiled(compiled.into()),
        }
    }
}

impl TryFrom<UiInstruction> for Instruction {
    type Error = RpcError;

    fn try_from(instruction: UiInstruction) -> Result<Self, Self::Error> {
        match instruction {
            UiInstruction::Compiled(compiled) => Ok(Self::Compiled(compiled.into())),
            UiInstruction::Parsed(_) => Err(RpcError::ValidationError(
                "Parsed instructions are not supported".to_string(),
            )),
        }
    }
}

/// Represents a compiled [instruction](https://solana.com/de/docs/references/terminology#instruction)
/// as part of a Solana transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct CompiledInstruction {
    /// Index into the transaction `message.accountKeys` array (see the transaction
    /// [JSON structure](https://solana.com/de/docs/rpc/json-structures#transactions)) indicating
    /// the program account that executes this instruction.
    #[serde(rename = "programIdIndex")]
    pub program_id_index: u8,
    /// List of ordered indices into the transaction `message.accountKeys` array (see the transaction
    /// [JSON structure](https://solana.com/de/docs/rpc/json-structures#transactions)) indicating
    /// which accounts to pass to the program.
    pub accounts: Vec<u8>,
    /// The program input data encoded in a base-58 string.
    pub data: String,
    /// The stack height at which this instruction was invoked during cross-program execution.
    #[serde(rename = "stackHeight")]
    pub stack_height: Option<u32>,
}

impl From<UiCompiledInstruction> for CompiledInstruction {
    fn from(instruction: UiCompiledInstruction) -> Self {
        Self {
            program_id_index: instruction.program_id_index,
            accounts: instruction.accounts,
            data: instruction.data,
            stack_height: instruction.stack_height,
        }
    }
}

impl From<CompiledInstruction> for UiCompiledInstruction {
    fn from(instruction: CompiledInstruction) -> Self {
        Self {
            program_id_index: instruction.program_id_index,
            accounts: instruction.accounts,
            data: instruction.data,
            stack_height: instruction.stack_height,
        }
    }
}
