//! Codegen

use std::ptr::null_mut;

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildCall2, LLVMBuildGlobalStringPtr,
        LLVMBuildRet, LLVMBuildRetVoid, LLVMCreateBuilder, LLVMDisposeBuilder, LLVMDumpModule,
        LLVMFunctionType, LLVMInt32Type, LLVMInt8Type, LLVMModuleCreateWithName,
        LLVMPositionBuilderAtEnd, LLVMVoidType,
    },
    LLVMModule,
};

use crate::node::{LiteralExpr, Stmt};

macro_rules! cstr {
    ($s:expr) => {
        std::ffi::CString::new($s).unwrap().as_ptr()
    };
}

// MODULES ARE IGNORED FOR NOW
pub unsafe fn codegen(nodes: Vec<Stmt>) {
    let printf_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 1);
    let main_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 0);

    let mut nodes = nodes.into_iter().peekable();

    let main_mod = LLVMModuleCreateWithName(cstr!("main"));

    let builder = LLVMCreateBuilder();

    // add print function
    let printf = LLVMAddFunction(main_mod, cstr!("printf"), printf_ty);
    pub struct Function {
        pub ty: *mut llvm_sys::LLVMType,
        pub func: *mut llvm_sys::LLVMValue,
    }

    let mut functions: Vec<Function> = Vec::new();
    while let Some(node) = nodes.peek() {
        println!("{:?}", node);
        match node {
            Stmt::Function { name, nodes } => {
                let LiteralExpr::String(func_name) = name;
                let func_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 1);

                let func = LLVMAddFunction(main_mod, cstr!(func_name.as_bytes()), func_ty);

                let func_block = LLVMAppendBasicBlock(func, cstr!("entry"));
                LLVMPositionBuilderAtEnd(builder, func_block);

                let call = LLVMBuildCall2(
                    builder,
                    printf_ty,
                    printf,
                    [].as_mut_ptr(),
                    0 as u32,
                    cstr!("func_printf_call"),
                );

                LLVMBuildRet(builder, call);
                functions.push(Function { ty: func_ty, func });
                // Later on there will be actual eval here or something like that but for no there is not. Hold the L.
            }
            _ => {}
        }
        nodes.next();
    }

    let main_func = LLVMAddFunction(main_mod, cstr!("main"), main_ty);

    let main_func_block = LLVMAppendBasicBlock(main_func, cstr!("entry"));
    LLVMPositionBuilderAtEnd(builder, main_func_block);

    let strptr =
        LLVMBuildGlobalStringPtr(builder, cstr!("Snout is cute"), cstr!("only_global_var"));
    let mut call_args = [strptr];

    let mut calls = Vec::new();

    for func in functions {
        let call = LLVMBuildCall2(
            builder,
            func.ty,
            func.func,
            call_args.as_mut_ptr(),
            call_args.len() as u32,
            cstr!("call_anon_func"),
        );
        calls.push(call);
    }

    if let Some(last) = calls.last() {
        LLVMBuildRet(builder, *last);
    }

    LLVMDumpModule(main_mod);
    LLVMVerifyModule(
        main_mod,
        LLVMVerifierFailureAction::LLVMAbortProcessAction,
        std::ptr::null_mut(),
    );
    llvm_sys::core::LLVMPrintModuleToFile(main_mod, cstr!("cbt.ll"), null_mut());
    LLVMDisposeBuilder(builder);
}
