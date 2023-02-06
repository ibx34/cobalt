use llvm_sys::{
    prelude::{LLVMBasicBlockRef, LLVMValueRef},
    LLVMType,
};

pub struct Function {
    pub entry: LLVMBasicBlockRef,
    pub ret: LLVMBasicBlockRef,
    pub ty: *mut LLVMType,
    pub func: LLVMValueRef,
}
