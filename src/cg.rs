//! Codegen

pub mod func;
pub mod var;

use std::{collections::HashMap, hash::Hash, iter::Peekable, ptr::null_mut};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMArrayType, LLVMBuildAlloca, LLVMBuildCall2,
        LLVMBuildGlobalStringPtr, LLVMBuildLoad2, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore,
        LLVMConstString, LLVMContextCreate, LLVMCreateBuilder, LLVMDisposeBuilder, LLVMDumpModule,
        LLVMDumpValue, LLVMFunctionType, LLVMInt32Type, LLVMInt8Type, LLVMModuleCreateWithName,
        LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMVectorType, LLVMVoidType,
    },
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMDIBuilderRef, LLVMModuleRef, LLVMValueRef},
    LLVMContext, LLVMModule, LLVMType,
};

use crate::node::{Expr, LiteralExpr, Stmt, VariableType};

use self::{func::Function, var::Variable};

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
    pub variables: HashMap<String, Variable>,
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
            variables: HashMap::new(),
            idx: 0,
        }
    }

    pub unsafe fn setup_main_module(&mut self) {
        let main_module = LLVMModuleCreateWithName(cstr!("main"));

        let printf_ty = LLVMFunctionType(LLVMInt32Type(), [].as_mut_ptr(), 0, 1);
        let printf = LLVMAddFunction(main_module, cstr!("printf"), printf_ty);

        self.functions.insert(
            "main-printf".to_string(),
            Function {
                entry: None,
                ret: None,
                ty: printf_ty,
                func: printf,
            },
        );

        self.cur_module = Some(main_module);
    }

    pub unsafe fn visit_block(&mut self, func: &Function, block: Vec<Box<Stmt>>) {
        let mut peekable = block.into_iter().map(|e| *e).peekable();
        while let Some(stmt) = peekable.peek() {
            match stmt {
                Stmt::Expr(expr) => {
                    match expr {
                        Expr::Call(func_call) => {
                            let Expr::Literal(LiteralExpr::String(name)) = *func_call.func.clone() else {
                                panic!("Expected string literal");
                            };
                            let Some(function) = self.functions.get(&format!("main-{name}")) else {
                            panic!("Function {name} not defined.");
                        };
                            LLVMPositionBuilderAtEnd(self.builder, func.entry.unwrap());

                            let mut arguments: Vec<LLVMValueRef> =
                                if let Some(args) = &func_call.args {
                                    args.clone()
                                .into_iter()
                                .map(|e| match *e {
                                    Expr::Literal(lit) => {
                                        let LiteralExpr::String(name) = lit;
                                        let Some(variable_ptr) = self.variables.get(&name) else {
                                            panic!("Undefined var.");
                                        };
                                        variable_ptr.ptr
                                        // doesn't like to load stuff?
                                        // LLVMBuildLoad2(self.builder, variable_ptr.ty, variable_ptr.ptr, cstr!("".as_bytes()))
                                    }
                                    _ => unimplemented!(),
                                })
                                .collect::<Vec<LLVMValueRef>>()
                                } else {
                                    Vec::new()
                                };

                            LLVMBuildCall2(
                                self.builder,
                                function.ty,
                                function.func,
                                arguments.as_mut_ptr(),
                                arguments.len() as u32,
                                cstr!("".as_bytes()),
                            );
                        }
                        _ => {}
                    }
                }
                Stmt::Variable { name, ty, value } => {
                    let value = match value {
                        Some(inner) => match inner {
                            Expr::Literal(literal) => match literal {
                                LiteralExpr::String(value) => value,
                            },
                            _ => todo!(),
                        },
                        None => todo!(),
                    };
                    let var_type = match ty {
                        VariableType::String => LLVMArrayType(
                            LLVMInt8Type(),
                            std::mem::size_of_val(value.as_bytes()) as u32,
                        ),
                    };
                    LLVMPositionBuilderAtEnd(self.builder, func.entry.unwrap());
                    let alloc = LLVMBuildAlloca(self.builder, var_type, cstr!(name.as_bytes()));
                    let val = LLVMConstString(
                        cstr!(value.as_bytes()),
                        std::mem::size_of_val(value.as_bytes()) as u32,
                        1,
                    );
                    LLVMBuildStore(self.builder, val, alloc);
                    self.variables.insert(
                        name.to_string(),
                        Variable {
                            ptr: alloc,
                            ty: var_type,
                        },
                    );
                }
                _ => {}
            };
            peekable.next();
        }
    }

    pub unsafe fn visit_fn(&mut self, func: Stmt) {
        if let Some(current_module) = self.cur_module {
            let Stmt::Function { name, nodes } = func else {
                panic!("Not a function");
            };
            let LiteralExpr::String(name) = name;
            let main_ty = LLVMFunctionType(LLVMVoidType(), [].as_mut_ptr(), 0, 0);
            let main_func = LLVMAddFunction(current_module, cstr!(name.as_bytes()), main_ty);

            let entry = LLVMAppendBasicBlock(main_func, cstr!("entry"));
            // let ret = LLVMAppendBasicBlock(main_func, cstr!("return"));

            let function = Function {
                entry: Some(entry),
                ret: None,
                ty: main_ty,
                func: main_func,
            };

            match *nodes {
                Stmt::Block(stmts) => self.visit_block(&function, stmts),
                _ => panic!("Expected a block?"),
            }

            LLVMPositionBuilderAtEnd(self.builder, function.entry.unwrap());
            LLVMBuildRetVoid(self.builder);

            self.functions.insert(format!("main-{}", name), function);
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
