//! Codegen

pub mod func;
pub mod var;

use std::{any::Any, collections::HashMap, hash::Hash, iter::Peekable, ptr::null_mut};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMArrayType, LLVMBuildAlloca, LLVMBuildBitCast,
        LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildFCmp, LLVMBuildGlobalStringPtr, LLVMBuildICmp,
        LLVMBuildLoad2, LLVMBuildPtrDiff2, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore,
        LLVMConstInt, LLVMConstString, LLVMContextCreate, LLVMCreateBuilder, LLVMCreatePassManager,
        LLVMDisposeBuilder, LLVMDumpModule, LLVMDumpValue, LLVMFunctionType, LLVMGetArrayLength,
        LLVMGetPointerAddressSpace, LLVMGlobalGetValueType, LLVMInt16Type, LLVMInt1Type,
        LLVMInt32Type, LLVMInt8Type, LLVMModuleCreateWithName, LLVMPointerType,
        LLVMPositionBuilderAtEnd, LLVMPrintModuleToFile, LLVMRunPassManager, LLVMVectorType,
        LLVMVoidType,
    },
    prelude::{
        LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMDIBuilderRef, LLVMModuleRef,
        LLVMTypeRef, LLVMValueRef,
    },
    target_machine::LLVMCodeGenOptLevel,
    transforms::pass_manager_builder::{
        LLVMPassManagerBuilderCreate, LLVMPassManagerBuilderDispose,
        LLVMPassManagerBuilderPopulateFunctionPassManager,
        LLVMPassManagerBuilderPopulateModulePassManager, LLVMPassManagerBuilderSetOptLevel,
    },
    LLVMContext, LLVMIntPredicate, LLVMModule, LLVMRealPredicate, LLVMType,
};

use crate::node::{BinaryOperators, Expr, LiteralExpr, Stmt, VariableType};

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
                blocks: Vec::new(),
            },
        );

        let strcmp_arg_tys = LLVMPointerType(LLVMInt8Type(), 0);
        let strcmp_ty = LLVMFunctionType(
            LLVMInt1Type(),
            [strcmp_arg_tys, strcmp_arg_tys].as_mut_ptr(),
            2,
            0,
        );
        let strcmp = LLVMAddFunction(main_module, cstr!("strcmp"), strcmp_ty);

        self.functions.insert(
            "main-strcmp".to_string(),
            Function {
                entry: None,
                ret: None,
                ty: strcmp_ty,
                func: strcmp,
                blocks: Vec::new(),
            },
        );

        self.cur_module = Some(main_module);
    }

    pub unsafe fn build_str_ptr(&self, value: String) -> (LLVMValueRef, u32, LLVMTypeRef) {
        let mut value = value.to_owned();
        value.push('\0');

        let size = std::mem::size_of_val(value.as_bytes()) as u32;
        let ty = LLVMArrayType(LLVMInt8Type(), size);

        let ptr = LLVMBuildAlloca(self.builder, ty, cstr!(""));
        let val = LLVMConstString(value.as_bytes().as_ptr() as *const i8, size, 1);
        LLVMBuildStore(self.builder, val, ptr);
        (ptr, size, ty)
    }

    pub unsafe fn build_int_ptr(
        &self,
        ty: LLVMTypeRef,
        int: libc::c_ulonglong,
    ) -> (LLVMValueRef, u32, LLVMTypeRef) {
        let ptr = LLVMBuildAlloca(self.builder, ty, cstr!(""));
        let val = LLVMConstInt(ty, int, 1);
        LLVMBuildStore(self.builder, val, ptr);
        (ptr, 1, ty)
    }

    pub unsafe fn create_string_var(&mut self, name: &str, value: String) -> Variable {
        let (ptr, size, ty) = self.build_str_ptr(value);
        let variable = Variable { size, ptr, ty };
        self.variables
            .insert(name.to_owned(), Variable { size, ptr, ty });
        variable
    }

    pub unsafe fn visit_block(
        &mut self,
        func: &Function,
        specific_bb: Option<LLVMBasicBlockRef>,
        block: Vec<Box<Stmt>>,
    ) {
        let mut peekable = block.into_iter().map(|e| *e).peekable();
        while let Some(stmt) = peekable.peek() {
            match stmt {
                Stmt::Condition(cond) => {
                    let Expr::BinaryOp(inner_cond) = *cond.clone().condition else {
                        panic!("Incorrect expr type");
                    };

                    let generate_tmp_val_ptr = |expr: Expr| match expr {
                        Expr::Literal(LiteralExpr::String(str)) => Some(
                            self.variables
                                .get(&str)
                                .and_then(|v| Some((v.ptr, v.size, v.ty)))
                                .unwrap_or(self.build_str_ptr(str)),
                        ),
                        Expr::Literal(LiteralExpr::Bool(bool)) => {
                            Some(self.build_int_ptr(LLVMInt1Type(), bool.into()))
                        }
                        _ => None,
                    };

                    let (left, size1, ty1) = generate_tmp_val_ptr(*inner_cond.l).unwrap();
                    let (right, size2, ty2) = generate_tmp_val_ptr(*inner_cond.r).unwrap();

                    if ty1 != ty2 {
                        panic!("Missmatched types");
                    }

                    //let icmp = LLVMBuildICmp(self.builder, op_ty, left, right, cstr!("if-cond"));
                    if let Some(strcmp) = self.functions.get("main-strcmp") {
                        let mut arguments = vec![
                            LLVMBuildBitCast(
                                self.builder,
                                left,
                                LLVMPointerType(LLVMInt8Type(), 0),
                                cstr!(""),
                            ),
                            LLVMBuildBitCast(
                                self.builder,
                                right,
                                LLVMPointerType(LLVMInt8Type(), 0),
                                cstr!(""),
                            ),
                        ];

                        let strcmp_call = LLVMBuildCall2(
                            self.builder,
                            strcmp.ty,
                            strcmp.func,
                            arguments.as_mut_ptr(),
                            arguments.len() as u32,
                            cstr!("".as_bytes()),
                        );

                        let zero = LLVMBuildAlloca(self.builder, LLVMInt1Type(), cstr!(""));
                        LLVMBuildStore(self.builder, LLVMConstInt(LLVMInt1Type(), 0, 1), zero);

                        let zero = LLVMBuildLoad2(self.builder, LLVMInt1Type(), zero, cstr!(""));

                        let icmp = LLVMBuildICmp(
                            self.builder,
                            LLVMIntPredicate::LLVMIntEQ,
                            strcmp_call,
                            zero,
                            cstr!(""),
                        );

                        let then = LLVMAppendBasicBlock(func.func, cstr!("condition"));
                        let r#else = LLVMAppendBasicBlock(func.func, cstr!("else"));

                        LLVMPositionBuilderAtEnd(self.builder, r#else);

                        LLVMBuildRetVoid(self.builder);

                        LLVMPositionBuilderAtEnd(self.builder, then);

                        if let Stmt::Block(block) = *cond.clone().then {
                            self.visit_block(func, Some(then), block)
                        }
                        LLVMBuildRetVoid(self.builder);
                        LLVMPositionBuilderAtEnd(
                            self.builder,
                            specific_bb.unwrap_or(func.entry.unwrap()),
                        );
                        LLVMBuildCondBr(self.builder, icmp, then, r#else);
                    }
                }
                Stmt::Expr(expr) => {
                    match expr {
                        Expr::Call(func_call) => {
                            let Expr::Literal(LiteralExpr::String(name)) = *func_call.func.clone() else {
                                panic!("Expected string literal");
                            };
                            let Some(function) = self.functions.get(&format!("main-{name}")) else {
                            panic!("Function {name} not defined.");
                        };
                            LLVMPositionBuilderAtEnd(
                                self.builder,
                                specific_bb.unwrap_or(func.entry.unwrap()),
                            );

                            let mut arguments: Vec<LLVMValueRef> =
                                if let Some(args) = &func_call.args {
                                    args.clone()
                                .into_iter()
                                .map(|e| match *e {
                                    Expr::Literal(lit) => {
                                        let LiteralExpr::String(name) = lit else {
                                            panic!("Incorrect literal")
                                        };
                                        let Some(variable_ptr) = self.variables.get(&name) else {
                                            panic!("Undefined var {:?}.", name);
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
                Stmt::Variable { name, ty, value } => self.visit_var(
                    func,
                    Some(specific_bb.unwrap_or(func.entry.unwrap())),
                    stmt.to_owned(),
                ),
                _ => {}
            };
            peekable.next();
        }
    }

    pub unsafe fn visit_var(
        &mut self,
        func: &Function,
        specific_bb: Option<LLVMBasicBlockRef>,
        variable: Stmt,
    ) {
        if let Stmt::Variable { name, ty, value } = variable {
            let var_type = match ty {
                VariableType::String => {
                    let Some(Expr::Literal(LiteralExpr::String(value))) = value.clone() else {
                        panic!("Expected a string literal.");
                    };
                    LLVMArrayType(
                        LLVMInt8Type(),
                        std::mem::size_of_val(value.as_bytes()) as u32 + 1,
                    )
                }
                VariableType::Bool => {
                    // Booleans are a single int: 1 or 0
                    LLVMInt1Type()
                }
            };

            LLVMPositionBuilderAtEnd(self.builder, specific_bb.unwrap_or(func.entry.unwrap()));
            let alloc = LLVMBuildAlloca(self.builder, var_type, cstr!(name.as_bytes()));

            let var = match value {
                Some(inner) => match inner {
                    Expr::Literal(LiteralExpr::String(string)) => {
                        let mut value = string.to_owned();
                        value.push('\0');

                        let val = LLVMConstString(
                            value.as_bytes().as_ptr() as *const i8,
                            std::mem::size_of_val(value.as_bytes()) as u32,
                            1,
                        );
                        LLVMBuildStore(self.builder, val, alloc);
                        Variable {
                            size: std::mem::size_of_val(value.as_bytes()) as u32,
                            ptr: alloc,
                            ty: var_type,
                        }
                    }
                    Expr::Literal(LiteralExpr::Bool(bool)) => {
                        LLVMBuildStore(
                            self.builder,
                            LLVMConstInt(LLVMInt1Type(), bool.into(), 1),
                            alloc,
                        );
                        Variable {
                            size: 1,
                            ptr: alloc,
                            ty: var_type,
                        }
                    }
                    _ => todo!(),
                },
                None => todo!(),
            };

            self.variables.insert(name.to_string(), var);
        }
    }

    pub unsafe fn visit_fn(&mut self, func: Stmt) {
        if let Some(current_module) = self.cur_module {
            let Stmt::Function { name, nodes } = func else {
                panic!("Not a function");
            };
            let LiteralExpr::String(name) = name else {
                panic!("Incorrect literal")
            };
            let main_ty = LLVMFunctionType(LLVMVoidType(), [].as_mut_ptr(), 0, 0);
            let main_func = LLVMAddFunction(current_module, cstr!(name.as_bytes()), main_ty);

            let entry = LLVMAppendBasicBlock(main_func, cstr!("entry"));
            // let ret = LLVMAppendBasicBlock(main_func, cstr!("return"));

            let mut function = Function {
                entry: Some(entry),
                ret: None,
                ty: main_ty,
                func: main_func,
                blocks: Vec::new(),
            };

            match *nodes {
                Stmt::Block(stmts) => self.visit_block(&function, None, stmts),
                _ => panic!("Expected a block?"),
            }

            LLVMPositionBuilderAtEnd(self.builder, function.entry.unwrap());
            if name == "main" {
                LLVMBuildRetVoid(self.builder);
            }

            self.functions.insert(format!("main-{}", name), function);
        }
    }

    pub fn advance(&mut self) -> Option<Stmt> {
        self.idx += 1;
        self.stmts.next()
    }

    pub unsafe fn verify_and_dump(&self) {
        if let Some(current_module) = self.cur_module {
            let pm = LLVMCreatePassManager();
            let pmb = LLVMPassManagerBuilderCreate();
            LLVMPassManagerBuilderSetOptLevel(
                pmb,
                LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive as u32,
            );
            LLVMPassManagerBuilderPopulateFunctionPassManager(pmb, pm);
            LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);

            LLVMVerifyModule(
                current_module,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                std::ptr::null_mut(),
            );

            LLVMRunPassManager(pm, current_module);

            LLVMPrintModuleToFile(current_module, cstr!("cbt.ll"), null_mut());

            LLVMPassManagerBuilderDispose(pmb);
            LLVMDisposeBuilder(self.builder);
        }
    }
}
