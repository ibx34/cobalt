use llvm_sys::{
    prelude::{LLVMBasicBlockRef, LLVMValueRef},
    LLVMType,
};

pub struct Function {
    pub entry: Option<LLVMBasicBlockRef>,
    pub ret: Option<LLVMBasicBlockRef>,
    pub blocks: Vec<LLVMBasicBlockRef>,
    pub ty: *mut LLVMType,
    pub func: LLVMValueRef,
}
