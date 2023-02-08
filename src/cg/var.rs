use llvm_sys::{
    prelude::{LLVMBasicBlockRef, LLVMValueRef},
    LLVMType,
};

pub struct Variable {
    pub ptr: LLVMValueRef,
    pub ty: *mut LLVMType,
}
