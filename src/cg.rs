//! Codegen

pub mod func;

use std::{collections::HashMap, hash::Hash, iter::Peekable, ptr::null_mut};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMArrayType, LLVMBuildAlloca, LLVMBuildCall2,
        LLVMBuildGlobalStringPtr, LLVMBuildRet, LLVMBuildRetVoid, LLVMContextCreate,
        LLVMCreateBuilder, LLVMDisposeBuilder, LLVMDumpModule, LLVMFunctionType, LLVMInt32Type,
        LLVMInt8Type, LLVMModuleCreateWithName, LLVMPositionBuilderAtEnd, LLVMVoidType,
    },
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMDIBuilderRef, LLVMModuleRef, LLVMValueRef},
    LLVMContext, LLVMModule, LLVMType,
};

use crate::node::{Expr, LiteralExpr, Stmt, VariableType};

use self::func::Function;

macro_rules! cstr {
    ($s:expr) => {
        std::ffi::CString::new($s).unwrap().as_ptr()
    };
}

pub struct CodeGen<T>
where
    T: Iterator<Item = Stmt>,
{
    pub context: LLVMContextRef,
    pub builder: LLVMBuilderRef,
    pub cur_module: Option<LLVMModuleRef>,
    pub modules: Vec<LLVMModuleRef>,
    pub stmts: Peekable<T>,
    // Hashmap key is `{module}-{func-name}`
    pub functions: HashMap<String, Function>,
    pub idx: usize,
}

impl<T> CodeGen<T>
where
    T: Iterator<Item = Stmt>,
{
    pub unsafe fn init(stmts: Peekable<T>) -> Self {
        Self {
            builder: LLVMCreateBuilder(),
            context: LLVMContextCreate(),
            modules: Vec::new(),
            cur_module: None,
            stmts,
            functions: HashMap::new(),
            idx: 0,
        }
    }

    pub unsafe fn setup_main_module(&mut self) {
        self.cur_module = Some(LLVMModuleCreateWithName(cstr!("main")));
    }

    pub unsafe fn visit_block(&mut self, block: Vec<Stmt>) {}

    pub unsafe fn visit_fn(&mut self, func: &Stmt) {
        if let Some(current_module) = self.cur_module {
            let Stmt::Function { name, nodes } = func else {
                panic!("Not a function");
            };
            let LiteralExpr::String(name) = name;
            let main_ty = LLVMFunctionType(LLVMVoidType(), [].as_mut_ptr(), 0, 0);
            let main_func = LLVMAddFunction(current_module, cstr!(name.as_bytes()), main_ty);

            let entry = LLVMAppendBasicBlock(main_func, cstr!("entry"));

            LLVMPositionBuilderAtEnd(self.builder, entry);
            LLVMBuildRetVoid(self.builder);

            let ret = LLVMAppendBasicBlock(main_func, cstr!("return"));

            LLVMPositionBuilderAtEnd(self.builder, ret);
            LLVMBuildRetVoid(self.builder);

            self.functions.insert(
                format!("main-{}", name),
                Function {
                    entry,
                    ret,
                    ty: main_ty,
                    func: main_func,
                },
            );
        }
    }

    pub fn advance(&mut self) -> Option<Stmt> {
        self.idx += 1;
        self.stmts.next()
    }

    pub unsafe fn verify_and_dump(&self) {
        if let Some(current_module) = self.cur_module {
            LLVMDumpModule(current_module);
            LLVMVerifyModule(
                current_module,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                std::ptr::null_mut(),
            );
            llvm_sys::core::LLVMPrintModuleToFile(current_module, cstr!("cbt.ll"), null_mut());
            LLVMDisposeBuilder(self.builder);
        }
    }
}

// MODULES ARE IGNORED FOR NOW
// pub unsafe fn codegen(nodes: Vec<Stmt>) {
//     let printf_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 1);
//     let main_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 0);

//     let mut nodes = nodes.into_iter().peekable();

//     let main_mod = LLVMModuleCreateWithName(cstr!("main"));

//     let builder = LLVMCreateBuilder();

//     // add print function
//     let printf = LLVMAddFunction(main_mod, cstr!("printf"), printf_ty);
//     pub struct Function {
//         pub ty: *mut llvm_sys::LLVMType,
//         pub func: *mut llvm_sys::LLVMValue,
//     }

//     let mut functions: Vec<Function> = Vec::new();
//     let mut variables: Vec<LLVMValueRef> = Vec::new();
//     while let Some(node) = nodes.peek() {
//         match node {
//             Stmt::Variable { name, ty, value } => {
//                 if ty == &VariableType::String {
//                     let Some(Expr::Literal(LiteralExpr::String(value))) = value else {
//                         panic!("Variable expression does not match type.");
//                     };
//                     // let value = cstr!(value.as_bytes());
//                     let ptr = LLVMBuildAlloca(
//                         builder,
//                         LLVMArrayType(LLVMInt8Type(), 1),
//                         cstr!(name.as_bytes()),
//                     );
//                 }
//                 nodes.next();
//             }
//             Stmt::Function { name, nodes } => {
//                 let LiteralExpr::String(func_name) = name;
//                 let func_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 1);

//                 let func = LLVMAddFunction(main_mod, cstr!(func_name.as_bytes()), func_ty);

//                 let func_block = LLVMAppendBasicBlock(func, cstr!("entry"));
//                 LLVMPositionBuilderAtEnd(builder, func_block);

//                 let call = LLVMBuildCall2(
//                     builder,
//                     printf_ty,
//                     printf,
//                     [].as_mut_ptr(),
//                     0 as u32,
//                     cstr!("func_printf_call"),
//                 );

//                 LLVMBuildRet(builder, call);
//                 functions.push(Function { ty: func_ty, func });
//                 // Later on there will be actual eval here or something like that but for no there is not. Hold the L.
//             }
//             _ => {}
//         }
//         nodes.next();
//     }

//     let main_func = LLVMAddFunction(main_mod, cstr!("main"), main_ty);

//     let main_func_block = LLVMAppendBasicBlock(main_func, cstr!("entry"));
//     LLVMPositionBuilderAtEnd(builder, main_func_block);

//     let mut calls = Vec::new();
//     if functions.len() > 0 {
//         for func in functions {
//             let call = LLVMBuildCall2(
//                 builder,
//                 func.ty,
//                 func.func,
//                 variables.as_mut_ptr(),
//                 variables.len() as u32,
//                 cstr!("call_anon_func"),
//             );
//             calls.push(call);
//         }
//     }

//     if let Some(last) = calls.last() {
//         LLVMBuildRet(builder, *last);
//     }
//     LLVMDumpModule(main_mod);
//     LLVMVerifyModule(
//         main_mod,
//         LLVMVerifierFailureAction::LLVMAbortProcessAction,
//         std::ptr::null_mut(),
//     );
//     llvm_sys::core::LLVMPrintModuleToFile(main_mod, cstr!("cbt.ll"), null_mut());
//     LLVMDisposeBuilder(builder);
// }
