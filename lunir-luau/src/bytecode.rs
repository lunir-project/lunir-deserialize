use num_enum::TryFromPrimitive;
use lunir::il::Constant;

#[derive(Debug)]
pub struct LuauChunk {

}

#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    Nop = 0,
    Break = 1,

    LoadNil = 2,
    LoadBool = 3,
    LoadNumber = 4,
    LoadConstant = 5,
    Move = 6,

    GetGlobal = 7,
    SetGlobal = 8,

    GetUpvalue = 9,
    SetUpValue = 10,

    GetImport = 11,

    GetTable = 12,
    SetTable = 13,

    GetTableString = 14,
    SetTableString = 15,

    GetTableNumber = 16,
    SetTableNumber = 17,

    NewClosure = 18,

    Namecall = 19,
    Call = 20,
    Return = 21,

    Jump = 22,
    JumpBack = 23,
    JumpIf = 24,
    JumpIfNot = 25,

    JumpIfEqual = 26,

    JumpIfLessThanOrEqual = 27,
    JumpIfLessThan = 28,

    JumpIfNotEqual = 29,
    JumpIfNotLessThanOrEqual = 30,
    JumpIfNotLessThan = 31,

    Add = 32,
    Sub = 33,
    Mul = 34,
    Div = 35,
    Mod = 36,
    Pow = 37,

    AddConstant = 38,
    SubConstant = 39,
    MulConstant = 40,
    DivConstant = 41,
    ModConstant = 42,
    PowConstant = 43,

    And = 44,
    Or = 45,

    AndConstant = 46,
    OrConstant = 47,

    Concat = 48,

    Not = 49,
    Minus = 50,
    Length = 51,

    NewTable = 52,
    DupTable = 53,

    SetList = 54,

    NumericForPrep = 55,
    NumericForLoop = 56,

    GenericForLoop = 57,
    GenericForPrepINext = 58,
    GenericForPrepNext = 60,

    GetVarargs = 62,

    DupClosure = 63,

    PrepVarargs = 64,

    LoadConstantExtended = 65,
    JumpExtended = 66,

    FastCall = 67,

    Coverage = 68,
    Capture = 69,

    FastCallSingle = 72,
    FastCallDouble = 73,
    FastCallRegisterConstant = 74,

    GenericForPrep = 75,

    JumpExtendedIfConstantNil = 76,
    JumpExtendedIfConstantEqualBool = 77,

    JumpExtendedIfConstantEqualNumber = 78,
    JumpExtendedIfConstantEqualString = 79,

    DeprecatedForLoopINext = 59,
    DeprecatedForPrepNext = 61,
    DeprecatedJumpIfConstantEqual = 70,
    DeprecatedJumpIfConstantNotEqual = 71,
}

pub(crate) struct Instruction(u32);

impl Instruction {
    #[must_use = "The error case of the opcode must be handled as it may be invalid."]
    pub(crate) fn opcode(&self) -> Result<OpCode, ()> {
        OpCode::try_from((self.0 & 0xFF) as u8).map_err(|_| ())
    }

    pub(crate) fn operand_a(&self) -> u8 {
        (self.0 >> 8 & 0xFF) as u8
    }

    pub(crate) fn operand_b(&self) -> u8 {
        (self.0 >> 16 & 0xFF) as u8
    }

    pub(crate) fn operand_c(&self) -> u8 {
        (self.0 >> 24 & 0xFF) as u8
    }

    pub(crate) fn operand_d(&self) -> i16 {
        (self.0 >> 16) as i16
    }

    pub(crate) fn operand_e(&self) -> i32 {
        (self.0 >> 8) as i32
    }

    pub(crate) fn raw(&self) -> u32 {
        self.0
    }
}

// lemme make a test and confirm that this stuff works
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoding() {
        let jump = Instruction(0x10_00_FF_16);

        assert_eq!(jump.opcode().unwrap(), OpCode::Jump);
        assert_eq!(jump.operand_a(), 0xFF);
        assert_eq!(jump.operand_b(), 0x00);
        assert_eq!(jump.operand_c(), 0x10);
        assert_eq!(jump.operand_d(), 0x10_00);
    }

    #[test]
    #[should_panic]
    fn test_invalid_opcode() {
        let _ = Instruction(0x50_FF_8C_FF).opcode().unwrap();
    }
}
