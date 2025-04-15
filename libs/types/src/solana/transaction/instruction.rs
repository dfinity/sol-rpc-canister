use crate::RpcError;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use solana_transaction_status_client_types::{
    UiCompiledInstruction, UiInnerInstructions, UiInstruction,
};

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct InnerInstructions {
    pub index: u8,
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
                .map(TryInto::<Instruction>::try_into)
                .collect::<Result<Vec<Instruction>, Self::Error>>()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum Instruction {
    Compiled(CompiledInstruction),
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

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: String,
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
