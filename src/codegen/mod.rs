// src/codegen/mod.rs - UPDATED with complete control flow implementation
//! Code generation for the Eä programming language.
//! 
//! This module is responsible for transforming the AST into LLVM IR,
//! which can then be optimized and compiled to machine code.

use inkwell::{
    context::Context,
    module::Module,
    builder::Builder,
    values::{FunctionValue, BasicValueEnum, PointerValue, VectorValue, BasicValue},
    types::{BasicTypeEnum, BasicType, VectorType},
    basic_block::BasicBlock,
    OptimizationLevel,
    targets::{
        Target, TargetMachine, RelocMode, CodeModel, FileType, 
        InitializationConfig
    },
    AddressSpace,
    IntPredicate, FloatPredicate,
};
use std::collections::HashMap;
use std::path::Path;
use crate::ast::{Expr, Stmt, Literal, BinaryOp, UnaryOp, TypeAnnotation, SIMDExpr, SIMDVectorType, SIMDOperator};
use crate::error::{CompileError, Result};
use crate::type_system::{TypeChecker, EaType};

/// Code generator for the Eä programming language.
pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    optimization_level: OptimizationLevel,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// Creates a new code generator with the given context and module name.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
    let module = context.create_module(module_name);
    let builder = context.create_builder();
    
    let mut codegen = Self {
        context,
        module,
        builder,
        variables: HashMap::new(),
        functions: HashMap::new(),
        optimization_level: OptimizationLevel::Default,
    };
    
    // Add this line:
    codegen.add_builtin_functions();
    
    codegen
}
/// Adds built-in functions to the code generator
fn add_builtin_functions(&mut self) {
    // Add external printf function declaration
    let string_type = self.context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = self.context.i32_type().fn_type(&[string_type.into()], true); // variadic
    let printf_function = self.module.add_function("printf", printf_type, None);
    self.functions.insert("printf".to_string(), printf_function);
    
    // Add external puts function declaration (simpler for string printing)
    let puts_type = self.context.i32_type().fn_type(&[string_type.into()], false);
    let puts_function = self.module.add_function("puts", puts_type, None);
    self.functions.insert("puts".to_string(), puts_function);
    
    // Add print(string) -> void function
    let print_type = self.context.void_type().fn_type(&[string_type.into()], false);
    let print_function = self.module.add_function("print", print_type, None);
    self.functions.insert("print".to_string(), print_function);
    
    // Implement print(string) function
    let print_entry = self.context.append_basic_block(print_function, "entry");
    let current_block = self.builder.get_insert_block();
    
    self.builder.position_at_end(print_entry);
    let param = print_function.get_nth_param(0).unwrap();
    
    // Call puts to print the string
    let _call = self.builder.build_call(puts_function, &[param.into()], "puts_call");
    self.builder.build_return(None).unwrap();
    
    // Add print_i32(i32) -> void function
    let i32_type = self.context.i32_type();
    let print_i32_type = self.context.void_type().fn_type(&[i32_type.into()], false);
    let print_i32_function = self.module.add_function("print_i32", print_i32_type, None);
    self.functions.insert("print_i32".to_string(), print_i32_function);
    
    // Implement print_i32 function
    let print_i32_entry = self.context.append_basic_block(print_i32_function, "entry");
    self.builder.position_at_end(print_i32_entry);
    
    let i32_param = print_i32_function.get_nth_param(0).unwrap();
    let format_str = self.builder.build_global_string_ptr("%d\n", "i32_format").unwrap();
    
    let _printf_call = self.builder.build_call(
        printf_function, 
        &[format_str.as_pointer_value().into(), i32_param.into()], 
        "printf_call"
    );
    self.builder.build_return(None).unwrap();
    
    // Add print_f32(f32) -> void function
    let f32_type = self.context.f32_type();
    let print_f32_type = self.context.void_type().fn_type(&[f32_type.into()], false);
    let print_f32_function = self.module.add_function("print_f32", print_f32_type, None);
    self.functions.insert("print_f32".to_string(), print_f32_function);
    
    // Implement print_f32 function
    let print_f32_entry = self.context.append_basic_block(print_f32_function, "entry");
    self.builder.position_at_end(print_f32_entry);
    
    let f32_param = print_f32_function.get_nth_param(0).unwrap();
    // Convert f32 to f64 for printf (C varargs promote float to double)
    let f64_param = self.builder.build_float_ext(
        f32_param.into_float_value(), 
        self.context.f64_type(), 
        "f32_to_f64"
    ).unwrap();
    
    let f32_format_str = self.builder.build_global_string_ptr("%.6f\n", "f32_format").unwrap();
    
    let _printf_call = self.builder.build_call(
        printf_function, 
        &[f32_format_str.as_pointer_value().into(), f64_param.into()], 
        "printf_call"
    );
    self.builder.build_return(None).unwrap();
    
    // Restore previous position if there was one
    if let Some(block) = current_block {
        self.builder.position_at_end(block);
    }
}
    
    /// Sets the optimization level.
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }
    
    /// Compiles the AST into LLVM IR.
    pub fn compile_program(&mut self, program: &[Stmt]) -> Result<()> {
        // Initialize target for the current machine
        Self::initialize_native_target();
        
        // Generate code for each statement in the program
        for stmt in program {
            self.generate_statement(stmt)?;
        }
        
        Ok(())
    }
    
    /// Initializes LLVM target for the current machine.
    fn initialize_native_target() {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");
    }
    
    /// Generates code for a statement.
    fn generate_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::FunctionDeclaration { name, params, return_type, body } => {
                self.generate_function_declaration(name, params, return_type, body)
            },
            Stmt::VarDeclaration { name, type_annotation, initializer } => {
                self.generate_var_declaration(name, type_annotation, initializer)
            },
            Stmt::Expression(expr) => {
                // Generate code for the expression but discard the result
                self.generate_expression(expr)?;
                Ok(())
            },
            Stmt::Return(expr) => self.generate_return(expr),
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.generate_if_statement(condition, then_branch, else_branch)
            },
            Stmt::While { condition, body } => {
                self.generate_while_statement(condition, body)
            },
            Stmt::For { initializer, condition, increment, body } => {
                self.generate_for_statement(initializer, condition, increment, body)
            },
        }
    }
    
    /// Generates code for an if statement using LLVM basic blocks.
    fn generate_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>
    ) -> Result<()> {
        let function = self.builder.get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| CompileError::codegen_error(
                "If statement outside of function context".to_string(),
                None
            ))?;

        // Create basic blocks for the if statement
        let then_block = self.context.append_basic_block(function, "if_then");
        let else_block = if else_branch.is_some() {
            Some(self.context.append_basic_block(function, "if_else"))
        } else {
            None
        };
        let merge_block = self.context.append_basic_block(function, "if_merge");

        // Generate condition
        let condition_value = self.generate_expression(condition)?;
        let condition_bool = self.convert_to_bool(condition_value)?;

        // Create conditional branch
        match else_block {
            Some(else_bb) => {
                self.builder.build_conditional_branch(condition_bool, then_block, else_bb)
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build conditional branch: {:?}", e),
                        None
                    ))?;
            },
            None => {
                self.builder.build_conditional_branch(condition_bool, then_block, merge_block)
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build conditional branch: {:?}", e),
                        None
                    ))?;
            }
        }

        // Generate then branch
        self.builder.position_at_end(then_block);
        self.generate_statement(then_branch)?;
        
        // Add branch to merge block if then branch doesn't already terminate
        if !self.block_has_terminator(then_block) {
            self.builder.build_unconditional_branch(merge_block)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to build branch to merge: {:?}", e),
                    None
                ))?;
        }

        // Generate else branch if it exists
        if let Some(else_stmt) = else_branch {
            let else_bb = else_block.unwrap();
            self.builder.position_at_end(else_bb);
            self.generate_statement(else_stmt)?;
            
            // Add branch to merge block if else branch doesn't already terminate
            if !self.block_has_terminator(else_bb) {
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build branch to merge: {:?}", e),
                        None
                    ))?;
            }
        }

        // Continue with merge block
        self.builder.position_at_end(merge_block);
        Ok(())
    }

    /// Generates code for a while loop using LLVM basic blocks.
    fn generate_while_statement(&mut self, condition: &Expr, body: &Box<Stmt>) -> Result<()> {
        let function = self.builder.get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| CompileError::codegen_error(
                "While statement outside of function context".to_string(),
                None
            ))?;

        // Create basic blocks for the while loop
        let loop_cond_block = self.context.append_basic_block(function, "while_cond");
        let loop_body_block = self.context.append_basic_block(function, "while_body");
        let loop_end_block = self.context.append_basic_block(function, "while_end");

        // Branch to condition check
        self.builder.build_unconditional_branch(loop_cond_block)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build branch to while condition: {:?}", e),
                None
            ))?;

        // Generate condition check
        self.builder.position_at_end(loop_cond_block);
        let condition_value = self.generate_expression(condition)?;
        let condition_bool = self.convert_to_bool(condition_value)?;

        // Conditional branch: if true go to body, if false go to end
        self.builder.build_conditional_branch(condition_bool, loop_body_block, loop_end_block)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build while conditional branch: {:?}", e),
                None
            ))?;

        // Generate loop body
        self.builder.position_at_end(loop_body_block);
        self.generate_statement(body)?;
        
        // Branch back to condition check if body doesn't already terminate
        if !self.block_has_terminator(loop_body_block) {
            self.builder.build_unconditional_branch(loop_cond_block)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to build branch back to condition: {:?}", e),
                    None
                ))?;
        }

        // Continue after the loop
        self.builder.position_at_end(loop_end_block);
        Ok(())
    }

    /// Generates code for a for loop using LLVM basic blocks.
    fn generate_for_statement(
        &mut self,
        initializer: &Option<Box<Stmt>>,
        condition: &Option<Expr>,
        increment: &Option<Expr>,
        body: &Box<Stmt>
    ) -> Result<()> {
        let function = self.builder.get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| CompileError::codegen_error(
                "For statement outside of function context".to_string(),
                None
            ))?;

        // Create basic blocks for the for loop
        let loop_cond_block = self.context.append_basic_block(function, "for_cond");
        let loop_body_block = self.context.append_basic_block(function, "for_body");
        let loop_inc_block = self.context.append_basic_block(function, "for_inc");
        let loop_end_block = self.context.append_basic_block(function, "for_end");

        // Generate initializer
        if let Some(init) = initializer {
            self.generate_statement(init)?;
        }

        // Branch to condition check
        self.builder.build_unconditional_branch(loop_cond_block)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build branch to for condition: {:?}", e),
                None
            ))?;

        // Generate condition check
        self.builder.position_at_end(loop_cond_block);
        if let Some(cond) = condition {
            let condition_value = self.generate_expression(cond)?;
            let condition_bool = self.convert_to_bool(condition_value)?;

            // Conditional branch: if true go to body, if false go to end
            self.builder.build_conditional_branch(condition_bool, loop_body_block, loop_end_block)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to build for conditional branch: {:?}", e),
                    None
                ))?;
        } else {
            // No condition means infinite loop, always go to body
            self.builder.build_unconditional_branch(loop_body_block)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to build unconditional branch to body: {:?}", e),
                    None
                ))?;
        }

        // Generate loop body
        self.builder.position_at_end(loop_body_block);
        self.generate_statement(body)?;
        
        // Branch to increment if body doesn't already terminate
        if !self.block_has_terminator(loop_body_block) {
            self.builder.build_unconditional_branch(loop_inc_block)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to build branch to increment: {:?}", e),
                    None
                ))?;
        }

        // Generate increment
        self.builder.position_at_end(loop_inc_block);
        if let Some(inc) = increment {
            self.generate_expression(inc)?;
        }

        // Branch back to condition check
        self.builder.build_unconditional_branch(loop_cond_block)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build branch back to condition: {:?}", e),
                None
            ))?;

        // Continue after the loop
        self.builder.position_at_end(loop_end_block);
        Ok(())
    }

    /// Converts a value to a boolean for use in conditional branches.
    fn convert_to_bool(&self, value: BasicValueEnum<'ctx>) -> Result<inkwell::values::IntValue<'ctx>> {
        match value {
            BasicValueEnum::IntValue(int_val) => {
                if int_val.get_type() == self.context.bool_type() {
                    Ok(int_val)
                } else {
                    // Convert integer to bool by comparing with 0
                    let zero = int_val.get_type().const_zero();
                    self.builder.build_int_compare(IntPredicate::NE, int_val, zero, "tobool")
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to convert to bool: {:?}", e),
                            None
                        ))
                }
            },
            BasicValueEnum::FloatValue(float_val) => {
                // Convert float to bool by comparing with 0.0
                let zero = float_val.get_type().const_zero();
                self.builder.build_float_compare(FloatPredicate::ONE, float_val, zero, "tobool")
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to convert float to bool: {:?}", e),
                        None
                    ))
            },
            _ => Err(CompileError::codegen_error(
                "Cannot convert value to boolean".to_string(),
                None
            ))
        }
    }

    /// Checks if a basic block has a terminator instruction.
    fn block_has_terminator(&self, block: BasicBlock<'ctx>) -> bool {
        block.get_terminator().is_some()
    }
    
    /// Generates code for a function declaration.
    fn generate_function_declaration(
        &mut self,
        name: &str,
        params: &[crate::ast::Parameter],
        return_type: &Option<TypeAnnotation>,
        body: &Box<Stmt>
    ) -> Result<()> {
        // Determine the return type
        let return_llvm_type: Option<BasicTypeEnum> = match return_type {
            Some(type_ann) => {
                match type_ann.name.as_str() {
                    "i32" => Some(self.context.i32_type().into()),
                    "i64" => Some(self.context.i64_type().into()),
                    "f32" => Some(self.context.f32_type().into()),
                    "f64" => Some(self.context.f64_type().into()),
                    "bool" => Some(self.context.bool_type().into()),
                    "string" => Some(self.context.i8_type().ptr_type(AddressSpace::default()).into()),
                    "()" => None, // void type
                    _ => {
                        return Err(CompileError::codegen_error(
                            format!("Unsupported return type: {}", type_ann.name),
                            None
                        ));
                    }
                }
            },
            None => None, // void type
        };
        
        // Determine parameter types
        let mut param_types = Vec::new();
        for param in params {
            let param_type = match param.type_annotation.name.as_str() {
                "i32" => self.context.i32_type().into(),
                "i64" => self.context.i64_type().into(),
                "f32" => self.context.f32_type().into(),
                "f64" => self.context.f64_type().into(),
                "bool" => self.context.bool_type().into(),
                "string" => self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                _ => {
                    return Err(CompileError::codegen_error(
                        format!("Unsupported parameter type: {}", param.type_annotation.name),
                        None
                    ));
                }
            };
            param_types.push(param_type);
        }
        
        // Create the function type
        let fn_type = match return_llvm_type {
            Some(ret_type) => ret_type.fn_type(&param_types, false),
            None => self.context.void_type().fn_type(&param_types, false),
        };
        
        // Create the function
        let function = self.module.add_function(name, fn_type, None);
        
        // Add the function to our function map
        self.functions.insert(name.to_string(), function);
        
        // Create a new basic block for the function body
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        // Create variable allocations for parameters
        let old_variables = self.variables.clone();
        self.variables.clear();
        
        for (i, param) in params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32)
                .ok_or_else(|| CompileError::codegen_error(
                    format!("Failed to get parameter {}", i),
                    None
                ))?;
            
            // Allocate space on the stack for the parameter
            let alloca = self.create_entry_block_alloca(function, &param.name, param_value.get_type())?;
            
            // Store the parameter value
            self.builder.build_store(alloca, param_value)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to store parameter: {:?}", e),
                    None
                ))?;
            
            // Add the variable to our map
            self.variables.insert(param.name.clone(), alloca);
        }
        
        // Generate code for the function body
        if let Stmt::Block(stmts) = &**body {
            for stmt in stmts {
                self.generate_statement(stmt)?;
            }
        } else {
            return Err(CompileError::codegen_error(
                "Function body must be a block statement".to_string(),
                None
            ));
        }
        
        // Add a return instruction if the function doesn't have one already
        if !self.block_has_terminator(self.builder.get_insert_block().unwrap()) {
            if return_llvm_type.is_none() {
                // Add a void return
                self.builder.build_return(None)
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build return: {:?}", e),
                        None
                    ))?;
            } else {
                // This is an error - function should have a return value
                return Err(CompileError::codegen_error(
                    format!("Function '{}' must return a value", name),
                    None
                ));
            }
        }
        
        // Restore the previous variable map
        self.variables = old_variables;
        
        Ok(())
    }
    
    /// Creates an allocation instruction in the entry block of the function.
    fn create_entry_block_alloca(
        &self,
        function: FunctionValue<'ctx>,
        name: &str,
        ty: BasicTypeEnum<'ctx>
    ) -> Result<PointerValue<'ctx>> {
        let builder = self.context.create_builder();
        let entry = function.get_first_basic_block().unwrap();
        
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        
        builder.build_alloca(ty, name)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build alloca: {:?}", e),
                None
            ))
    }
    
    /// Generates code for a variable declaration.
    fn generate_var_declaration(
        &mut self,
        name: &str,
        type_annotation: &Option<TypeAnnotation>,
        initializer: &Option<Expr>
    ) -> Result<()> {
        // Determine the variable type
        let var_type = if let Some(type_ann) = type_annotation {
            match type_ann.name.as_str() {
                "i32" => self.context.i32_type().into(),
                "i64" => self.context.i64_type().into(),
                "f32" => self.context.f32_type().into(),
                "f64" => self.context.f64_type().into(),
                "bool" => self.context.bool_type().into(),
                "string" => self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                _ => {
                    return Err(CompileError::codegen_error(
                        format!("Unsupported variable type: {}", type_ann.name),
                        None
                    ));
                }
            }
        } else if let Some(init) = initializer {
            // Infer the type from the initializer
            let init_value = self.generate_expression(init)?;
            init_value.get_type()
        } else {
            // No type annotation and no initializer - error
            return Err(CompileError::codegen_error(
                format!("Variable '{}' needs either a type annotation or an initializer", name),
                None
            ));
        };
        
        // Allocate space on the stack for the variable
        let function = self.builder.get_insert_block()
            .and_then(|block| block.get_parent())
            .ok_or_else(|| CompileError::codegen_error(
                "Variable declaration outside of function context".to_string(),
                None
            ))?;
        
        let alloca = self.create_entry_block_alloca(function, name, var_type)?;
        
        // Store the initial value if provided
        if let Some(init) = initializer {
            let init_value = self.generate_expression(init)?;
            
            // Check that the initializer type matches the variable type
            if init_value.get_type() != var_type {
                return Err(CompileError::codegen_error(
                    format!("Type mismatch in variable initialization: expected {:?}, got {:?}", 
                        var_type, init_value.get_type()),
                    None
                ));
            }
            
            self.builder.build_store(alloca, init_value)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to store variable: {:?}", e),
                    None
                ))?;
        }
        
        // Add the variable to our map
        self.variables.insert(name.to_string(), alloca);
        
        Ok(())
    }
    
    /// Generates code for a return statement.
    fn generate_return(&mut self, expr: &Option<Expr>) -> Result<()> {
        match expr {
            Some(e) => {
                let return_value = self.generate_expression(e)?;
                self.builder.build_return(Some(&return_value))
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build return: {:?}", e),
                        None
                    ))?;
            },
            None => {
                self.builder.build_return(None)
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build return: {:?}", e),
                        None
                    ))?;
            }
        }
        
        Ok(())
    }
    
    /// Generates code for an expression.
    fn generate_expression(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Variable(name) => self.generate_variable_access(name),
            Expr::Binary(left, op, right) => self.generate_binary_expression(left, op, right),
            Expr::Unary(op, expr) => self.generate_unary_expression(op, expr),
            Expr::Call(callee, args) => self.generate_function_call(callee, args),
            Expr::Grouping(expr) => self.generate_expression(expr),
            Expr::SIMD(simd_expr) => {
                // Comprehensive SIMD expression code generation
                self.generate_simd_expression(simd_expr)
            }
            // For now, we'll add placeholders for other expression types
            Expr::Index(_, _) => {
                // Placeholder for array indexing
                Err(CompileError::codegen_error(
                    "Array indexing not yet implemented".to_string(),
                    None
                ))
            },
            Expr::FieldAccess(_, _) => {
                // Placeholder for field access
                Err(CompileError::codegen_error(
                    "Field access not yet implemented".to_string(),
                    None
                ))
            },
        }
    }
    
    /// Generates code for a literal value.
    fn generate_literal(&mut self, literal: &Literal) -> Result<BasicValueEnum<'ctx>> {
        match literal {
            Literal::Integer(value) => {
                let int_type = self.context.i32_type();
                Ok(int_type.const_int(*value as u64, true).into())
            },
            Literal::Float(value) => {
                let float_type = self.context.f64_type();
                Ok(float_type.const_float(*value).into())
            },
            Literal::String(value) => {
                // Create a global string constant
                let string_value = self.builder.build_global_string_ptr(value, "string_literal")
                    .map_err(|e| CompileError::codegen_error(
                        format!("Failed to build string: {:?}", e),
                        None
                    ))?;
                Ok(string_value.as_pointer_value().into())
            },
            Literal::Boolean(value) => {
                let bool_type = self.context.bool_type();
                Ok(bool_type.const_int(*value as u64, false).into())
            },
            Literal::Vector { elements, vector_type } => {
                // Proper SIMD vector literal code generation
                if let Some(vtype) = vector_type {
                    // Convert literal elements to expressions and call SIMD vector generation
                    let element_exprs: Vec<Expr> = elements.iter()
                        .map(|lit| Expr::Literal(lit.clone()))
                        .collect();
                    
                    let vector_val = self.generate_simd_vector_literal(&element_exprs, vtype)?;
                    Ok(vector_val.into())
                } else {
                    Err(CompileError::codegen_error(
                        "Vector literal must have explicit type annotation".to_string(),
                        None
                    ))
                }
            }
        }
    }
    
    /// Generates code for a variable access.
    fn generate_variable_access(&self, name: &str) -> Result<BasicValueEnum<'ctx>> {
        if let Some(&ptr) = self.variables.get(name) {
            let load = self.builder.build_load(ptr, name)
                .map_err(|e| CompileError::codegen_error(
                    format!("Failed to load variable: {:?}", e),
                    None
                ))?;
            Ok(load)
        } else {
            Err(CompileError::codegen_error(
                format!("Variable '{}' not found", name),
                None
            ))
        }
    }
    
    /// Generates code for a binary expression.
fn generate_binary_expression(
    &mut self,
    left: &Box<Expr>,
    op: &BinaryOp,
    right: &Box<Expr>
) -> Result<BasicValueEnum<'ctx>> {
    // Handle assignment operations FIRST, before evaluating operands
    match op {
        BinaryOp::Assign => {
            // Special case: handle assignment without evaluating left side as value
            if let Expr::Variable(var_name) = &**left {
                let right_value = self.generate_expression(right)?;
                if let Some(&var_ptr) = self.variables.get(var_name) {
                    self.builder.build_store(var_ptr, right_value)
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to store assignment: {:?}", e),
                            None
                        ))?;
                    Ok(right_value) // Return the assigned value
                } else {
                    Err(CompileError::codegen_error(
                        format!("Variable '{}' not found for assignment", var_name),
                        None
                    ))
                }
            } else {
                Err(CompileError::codegen_error(
                    "Left side of assignment must be a variable".to_string(),
                    None
                ))
            }
        },
        BinaryOp::PlusAssign => {
            if let Expr::Variable(var_name) = &**left {
                let current_val = self.generate_variable_access(var_name)?;
                let right_val = self.generate_expression(right)?;
                
                // Handle both integer and float addition
                let sum = if let (BasicValueEnum::IntValue(curr_int), BasicValueEnum::IntValue(right_int)) = (current_val, right_val) {
                    self.builder.build_int_add(curr_int, right_int, "add_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build add_assign: {:?}", e),
                            None
                        ))?
                } else if let (BasicValueEnum::FloatValue(curr_float), BasicValueEnum::FloatValue(right_float)) = (current_val, right_val) {
                    self.builder.build_float_add(curr_float, right_float, "fadd_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build fadd_assign: {:?}", e),
                            None
                        ))?
                } else {
                    return Err(CompileError::codegen_error(
                        "Type mismatch in += operation".to_string(),
                        None
                    ));
                };
                
                if let Some(&var_ptr) = self.variables.get(var_name) {
                    self.builder.build_store(var_ptr, sum)
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to store add_assign: {:?}", e),
                            None
                        ))?;
                    Ok(sum)
                } else {
                    Err(CompileError::codegen_error(
                        format!("Variable '{}' not found for +=", var_name),
                        None
                    ))
                }
            } else {
                Err(CompileError::codegen_error(
                    "Left side of += must be a variable".to_string(),
                    None
                ))
            }
        },
        BinaryOp::MinusAssign => {
            if let Expr::Variable(var_name) = &**left {
                let current_val = self.generate_variable_access(var_name)?;
                let right_val = self.generate_expression(right)?;
                
                let diff = if let (BasicValueEnum::IntValue(curr_int), BasicValueEnum::IntValue(right_int)) = (current_val, right_val) {
                    self.builder.build_int_sub(curr_int, right_int, "sub_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build sub_assign: {:?}", e),
                            None
                        ))?
                } else if let (BasicValueEnum::FloatValue(curr_float), BasicValueEnum::FloatValue(right_float)) = (current_val, right_val) {
                    self.builder.build_float_sub(curr_float, right_float, "fsub_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build fsub_assign: {:?}", e),
                            None
                        ))?
                } else {
                    return Err(CompileError::codegen_error(
                        "Type mismatch in -= operation".to_string(),
                        None
                    ));
                };
                
                if let Some(&var_ptr) = self.variables.get(var_name) {
                    self.builder.build_store(var_ptr, diff)
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to store sub_assign: {:?}", e),
                            None
                        ))?;
                    Ok(diff)
                } else {
                    Err(CompileError::codegen_error(
                        format!("Variable '{}' not found for -=", var_name),
                        None
                    ))
                }
            } else {
                Err(CompileError::codegen_error(
                    "Left side of -= must be a variable".to_string(),
                    None
                ))
            }
        },
        BinaryOp::MultiplyAssign => {
            if let Expr::Variable(var_name) = &**left {
                let current_val = self.generate_variable_access(var_name)?;
                let right_val = self.generate_expression(right)?;
                
                let product = if let (BasicValueEnum::IntValue(curr_int), BasicValueEnum::IntValue(right_int)) = (current_val, right_val) {
                    self.builder.build_int_mul(curr_int, right_int, "mul_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build mul_assign: {:?}", e),
                            None
                        ))?
                } else if let (BasicValueEnum::FloatValue(curr_float), BasicValueEnum::FloatValue(right_float)) = (current_val, right_val) {
                    self.builder.build_float_mul(curr_float, right_float, "fmul_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build fmul_assign: {:?}", e),
                            None
                        ))?
                } else {
                    return Err(CompileError::codegen_error(
                        "Type mismatch in *= operation".to_string(),
                        None
                    ));
                };
                
                if let Some(&var_ptr) = self.variables.get(var_name) {
                    self.builder.build_store(var_ptr, product)
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to store mul_assign: {:?}", e),
                            None
                        ))?;
                    Ok(product)
                } else {
                    Err(CompileError::codegen_error(
                        format!("Variable '{}' not found for *=", var_name),
                        None
                    ))
                }
            } else {
                Err(CompileError::codegen_error(
                    "Left side of *= must be a variable".to_string(),
                    None
                ))
            }
        },
        BinaryOp::DivideAssign => {
            if let Expr::Variable(var_name) = &**left {
                let current_val = self.generate_variable_access(var_name)?;
                let right_val = self.generate_expression(right)?;
                
                let quotient = if let (BasicValueEnum::IntValue(curr_int), BasicValueEnum::IntValue(right_int)) = (current_val, right_val) {
                    self.builder.build_int_signed_div(curr_int, right_int, "div_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build div_assign: {:?}", e),
                            None
                        ))?
                } else if let (BasicValueEnum::FloatValue(curr_float), BasicValueEnum::FloatValue(right_float)) = (current_val, right_val) {
                    self.builder.build_float_div(curr_float, right_float, "fdiv_assign")
                        .map(|v| v.into())
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build fdiv_assign: {:?}", e),
                            None
                        ))?
                } else {
                    return Err(CompileError::codegen_error(
                        "Type mismatch in /= operation".to_string(),
                        None
                    ));
                };
                
                if let Some(&var_ptr) = self.variables.get(var_name) {
                    self.builder.build_store(var_ptr, quotient)
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to store div_assign: {:?}", e),
                            None
                        ))?;
                    Ok(quotient)
                } else {
                    Err(CompileError::codegen_error(
                        format!("Variable '{}' not found for /=", var_name),
                        None
                    ))
                }
            } else {
                Err(CompileError::codegen_error(
                    "Left side of /= must be a variable".to_string(),
                    None
                ))
            }
        },
        _ => {
            // For all other operations, evaluate both operands first
            let left_value = self.generate_expression(left)?;
            let right_value = self.generate_expression(right)?;
            
            // Handle operations based on operand types
            if let (BasicValueEnum::IntValue(left_int), BasicValueEnum::IntValue(right_int)) = (left_value, right_value) {
                match op {
                    BinaryOp::Add => {
                        let result = self.builder.build_int_add(left_int, right_int, "add")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build add: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Subtract => {
                        let result = self.builder.build_int_sub(left_int, right_int, "sub")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build sub: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Multiply => {
                        let result = self.builder.build_int_mul(left_int, right_int, "mul")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build mul: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Divide => {
                        let result = self.builder.build_int_signed_div(left_int, right_int, "div")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build div: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Less => {
                        let result = self.builder.build_int_compare(IntPredicate::SLT, left_int, right_int, "cmp_lt")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::LessEqual => {
                        let result = self.builder.build_int_compare(IntPredicate::SLE, left_int, right_int, "cmp_le")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Greater => {
                        let result = self.builder.build_int_compare(IntPredicate::SGT, left_int, right_int, "cmp_gt")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::GreaterEqual => {
                        let result = self.builder.build_int_compare(IntPredicate::SGE, left_int, right_int, "cmp_ge")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Equal => {
                        let result = self.builder.build_int_compare(IntPredicate::EQ, left_int, right_int, "cmp_eq")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::NotEqual => {
                        let result = self.builder.build_int_compare(IntPredicate::NE, left_int, right_int, "cmp_ne")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Modulo => {
                        let result = self.builder.build_int_signed_rem(left_int, right_int, "mod")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build mod: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::And => {
                        // Convert to boolean (i1) if needed, then perform logical AND
                        let left_bool = if left_int.get_type().get_bit_width() == 1 {
                            left_int
                        } else {
                            // Convert non-zero to true, zero to false
                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder.build_int_compare(IntPredicate::NE, left_int, zero, "to_bool")
                                .map_err(|e| CompileError::codegen_error(
                                    format!("Failed to convert to bool: {:?}", e),
                                    None
                                ))?
                        };
                        
                        let right_bool = if right_int.get_type().get_bit_width() == 1 {
                            right_int
                        } else {
                            // Convert non-zero to true, zero to false
                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder.build_int_compare(IntPredicate::NE, right_int, zero, "to_bool")
                                .map_err(|e| CompileError::codegen_error(
                                    format!("Failed to convert to bool: {:?}", e),
                                    None
                                ))?
                        };
                        
                        let result = self.builder.build_and(left_bool, right_bool, "and")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build logical AND: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Or => {
                        // Convert to boolean (i1) if needed, then perform logical OR
                        let left_bool = if left_int.get_type().get_bit_width() == 1 {
                            left_int
                        } else {
                            // Convert non-zero to true, zero to false
                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder.build_int_compare(IntPredicate::NE, left_int, zero, "to_bool")
                                .map_err(|e| CompileError::codegen_error(
                                    format!("Failed to convert to bool: {:?}", e),
                                    None
                                ))?
                        };
                        
                        let right_bool = if right_int.get_type().get_bit_width() == 1 {
                            right_int
                        } else {
                            // Convert non-zero to true, zero to false
                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder.build_int_compare(IntPredicate::NE, right_int, zero, "to_bool")
                                .map_err(|e| CompileError::codegen_error(
                                    format!("Failed to convert to bool: {:?}", e),
                                    None
                                ))?
                        };
                        
                        let result = self.builder.build_or(left_bool, right_bool, "or")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build logical OR: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    // For other operations, we'll return an error for now
                    _ => Err(CompileError::codegen_error(
                        format!("Binary operation {:?} not yet implemented for integers", op),
                        None
                    )),
                }
            } else if let (BasicValueEnum::FloatValue(left_float), BasicValueEnum::FloatValue(right_float)) = (left_value, right_value) {
                match op {
                    BinaryOp::Add => {
                        let result = self.builder.build_float_add(left_float, right_float, "fadd")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build fadd: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Subtract => {
                        let result = self.builder.build_float_sub(left_float, right_float, "fsub")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build fsub: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Multiply => {
                        let result = self.builder.build_float_mul(left_float, right_float, "fmul")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build fmul: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Divide => {
                        let result = self.builder.build_float_div(left_float, right_float, "fdiv")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build fdiv: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Less => {
                        let result = self.builder.build_float_compare(FloatPredicate::OLT, left_float, right_float, "fcmp_lt")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build float comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::LessEqual => {
                        let result = self.builder.build_float_compare(FloatPredicate::OLE, left_float, right_float, "fcmp_le")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build float comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Greater => {
                        let result = self.builder.build_float_compare(FloatPredicate::OGT, left_float, right_float, "fcmp_gt")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build float comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::GreaterEqual => {
                        let result = self.builder.build_float_compare(FloatPredicate::OGE, left_float, right_float, "fcmp_ge")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build float comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Equal => {
                        let result = self.builder.build_float_compare(FloatPredicate::OEQ, left_float, right_float, "fcmp_eq")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build float comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::NotEqual => {
                        let result = self.builder.build_float_compare(FloatPredicate::ONE, left_float, right_float, "fcmp_ne")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build float comparison: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::And => {
                        // Convert floats to boolean: non-zero is true, zero/NaN is false
                        let zero = self.context.f64_type().const_float(0.0);
                        let left_bool = self.builder.build_float_compare(FloatPredicate::ONE, left_float, zero, "float_to_bool")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to convert float to bool: {:?}", e),
                                None
                            ))?;
                        let right_bool = self.builder.build_float_compare(FloatPredicate::ONE, right_float, zero, "float_to_bool")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to convert float to bool: {:?}", e),
                                None
                            ))?;
                        
                        let result = self.builder.build_and(left_bool, right_bool, "float_and")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build logical AND for floats: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    BinaryOp::Or => {
                        // Convert floats to boolean: non-zero is true, zero/NaN is false
                        let zero = self.context.f64_type().const_float(0.0);
                        let left_bool = self.builder.build_float_compare(FloatPredicate::ONE, left_float, zero, "float_to_bool")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to convert float to bool: {:?}", e),
                                None
                            ))?;
                        let right_bool = self.builder.build_float_compare(FloatPredicate::ONE, right_float, zero, "float_to_bool")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to convert float to bool: {:?}", e),
                                None
                            ))?;
                        
                        let result = self.builder.build_or(left_bool, right_bool, "float_or")
                            .map_err(|e| CompileError::codegen_error(
                                format!("Failed to build logical OR for floats: {:?}", e),
                                None
                            ))?;
                        Ok(result.into())
                    },
                    // For other operations, we'll return an error for now
                    _ => Err(CompileError::codegen_error(
                        format!("Binary operation {:?} not yet implemented for floats", op),
                        None
                    )),
                }
            } else {
                // If we have mixed types or other types, we'll return an error for now
                Err(CompileError::codegen_error(
                    "Mixed types in binary operation not yet supported".to_string(),
                    None
                ))
            }
        }
    }
}
    
    /// Generates code for a unary expression.
    fn generate_unary_expression(
        &mut self,
        op: &UnaryOp,
        expr: &Box<Expr>
    ) -> Result<BasicValueEnum<'ctx>> {
        let expr_value = self.generate_expression(expr)?;
        
        match op {
            UnaryOp::Negate => {
                if let BasicValueEnum::IntValue(int_value) = expr_value {
                    let result = self.builder.build_int_neg(int_value, "neg")
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build neg: {:?}", e),
                            None
                        ))?;
                    Ok(result.into())
                } else if let BasicValueEnum::FloatValue(float_value) = expr_value {
                    let result = self.builder.build_float_neg(float_value, "fneg")
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build fneg: {:?}", e),
                            None
                        ))?;
                    Ok(result.into())
                } else {
                    Err(CompileError::codegen_error(
                        "Negation only supported for numeric types".to_string(),
                        None
                    ))
                }
            },
            UnaryOp::Not => {
                if let BasicValueEnum::IntValue(int_value) = expr_value {
                    let result = self.builder.build_not(int_value, "not")
                        .map_err(|e| CompileError::codegen_error(
                            format!("Failed to build not: {:?}", e),
                            None
                        ))?;
                    Ok(result.into())
                } else {
                    Err(CompileError::codegen_error(
                        "Logical not only supported for boolean types".to_string(),
                        None
                    ))
                }
            },
            // For other unary operations, we'll return an error for now
            _ => Err(CompileError::codegen_error(
                format!("Unary operation {:?} not yet implemented", op),
                None
            )),
        }
    }
    
    /// Generates code for a function call.
    fn generate_function_call(
        &mut self,
        callee: &Box<Expr>,
        args: &[Expr]
    ) -> Result<BasicValueEnum<'ctx>> {
        // For now, we'll only support direct function calls by name
        if let Expr::Variable(func_name) = &**callee {
            if let Some(&function) = self.functions.get(func_name) {
                // Generate code for each argument
                let mut arg_values = Vec::new();
                for arg in args {
                    let arg_value = self.generate_expression(arg)?;
                    arg_values.push(arg_value);
                }
                
                // Build the function call
                let call = self.builder.build_call(
                    function, 
                    &arg_values.iter().map(|v| (*v).into()).collect::<Vec<_>>(),
                    "call"
                ).map_err(|e| CompileError::codegen_error(
                    format!("Failed to build call: {:?}", e),
                    None
                ))?;
                
                // Get the return value (if any)
                if let Some(return_value) = call.try_as_basic_value().left() {
    Ok(return_value)
} else {
    // Function returns void - create a dummy value for statement context
    let dummy_value = self.context.i32_type().const_int(0, false);
    Ok(dummy_value.into())
}
            } else {
                Err(CompileError::codegen_error(
                    format!("Function '{}' not found", func_name),
                    None
                ))
            }
        } else {
            Err(CompileError::codegen_error(
                "Only direct function calls by name are supported".to_string(),
                None
            ))
        }
    }
    
    /// Writes the generated LLVM IR to a file.
    pub fn write_ir_to_file(&self, filename: &str) -> Result<()> {
        if self.module.print_to_file(filename).is_err() {
            return Err(CompileError::codegen_error(
                format!("Failed to write IR to file '{}'", filename),
                None
            ));
        }
        Ok(())
    }
    
    /// Returns the generated LLVM IR as a string.
    pub fn emit_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
    
    /// Returns a reference to the LLVM module for JIT execution.
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }
    
    /// Compiles the module to an object file.
    pub fn compile_to_object_file(&self, filename: &str) -> Result<()> {
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to create target: {}", e),
                None
            ))?;
            
        let machine = target.create_target_machine(
            &triple,
            "x86-64",
            "+avx2",
            self.optimization_level,
            RelocMode::Default,
            CodeModel::Default,
        ).ok_or_else(|| CompileError::codegen_error(
            "Failed to create target machine".to_string(),
            None
        ))?;
        
        let result = machine.write_to_file(
            &self.module, 
            FileType::Object, 
            Path::new(filename)
        );
        
        if let Err(e) = result {
            return Err(CompileError::codegen_error(
                format!("Failed to write object file: {}", e),
                None
            ));
        }
        
        Ok(())
    }
    
    /// Converts SIMD vector type to LLVM vector type.
    fn simd_type_to_llvm(&self, simd_type: &SIMDVectorType) -> Result<VectorType<'ctx>> {
        match simd_type {
            // Float vectors
            SIMDVectorType::F32x2 => Ok(self.context.f32_type().vec_type(2)),
            SIMDVectorType::F32x4 => Ok(self.context.f32_type().vec_type(4)),
            SIMDVectorType::F32x8 => Ok(self.context.f32_type().vec_type(8)),
            SIMDVectorType::F32x16 => Ok(self.context.f32_type().vec_type(16)),
            SIMDVectorType::F64x2 => Ok(self.context.f64_type().vec_type(2)),
            SIMDVectorType::F64x4 => Ok(self.context.f64_type().vec_type(4)),
            SIMDVectorType::F64x8 => Ok(self.context.f64_type().vec_type(8)),
            
            // Integer vectors
            SIMDVectorType::I32x2 => Ok(self.context.i32_type().vec_type(2)),
            SIMDVectorType::I32x4 => Ok(self.context.i32_type().vec_type(4)),
            SIMDVectorType::I32x8 => Ok(self.context.i32_type().vec_type(8)),
            SIMDVectorType::I32x16 => Ok(self.context.i32_type().vec_type(16)),
            SIMDVectorType::I64x2 => Ok(self.context.i64_type().vec_type(2)),
            SIMDVectorType::I64x4 => Ok(self.context.i64_type().vec_type(4)),
            SIMDVectorType::I64x8 => Ok(self.context.i64_type().vec_type(8)),
            SIMDVectorType::I16x4 => Ok(self.context.i16_type().vec_type(4)),
            SIMDVectorType::I16x8 => Ok(self.context.i16_type().vec_type(8)),
            SIMDVectorType::I16x16 => Ok(self.context.i16_type().vec_type(16)),
            SIMDVectorType::I16x32 => Ok(self.context.i16_type().vec_type(32)),
            SIMDVectorType::I8x8 => Ok(self.context.i8_type().vec_type(8)),
            SIMDVectorType::I8x16 => Ok(self.context.i8_type().vec_type(16)),
            SIMDVectorType::I8x32 => Ok(self.context.i8_type().vec_type(32)),
            SIMDVectorType::I8x64 => Ok(self.context.i8_type().vec_type(64)),
            
            // Unsigned integer vectors (LLVM treats as signed)
            SIMDVectorType::U32x4 => Ok(self.context.i32_type().vec_type(4)),
            SIMDVectorType::U32x8 => Ok(self.context.i32_type().vec_type(8)),
            SIMDVectorType::U16x8 => Ok(self.context.i16_type().vec_type(8)),
            SIMDVectorType::U16x16 => Ok(self.context.i16_type().vec_type(16)),
            SIMDVectorType::U8x16 => Ok(self.context.i8_type().vec_type(16)),
            SIMDVectorType::U8x32 => Ok(self.context.i8_type().vec_type(32)),
            
            // Mask types (represented as integer vectors in LLVM)
            SIMDVectorType::Mask8 => Ok(self.context.bool_type().vec_type(8)),
            SIMDVectorType::Mask16 => Ok(self.context.bool_type().vec_type(16)),
            SIMDVectorType::Mask32 => Ok(self.context.bool_type().vec_type(32)),
            SIMDVectorType::Mask64 => Ok(self.context.bool_type().vec_type(64)),
        }
    }
    
    /// Generates a SIMD element with the correct type for the target vector.
    fn generate_simd_element(
        &mut self,
        element: &Expr,
        vector_type: &SIMDVectorType
    ) -> Result<BasicValueEnum<'ctx>> {
        match element {
            Expr::Literal(Literal::Float(value)) => {
                // Generate float with the correct precision for the vector type
                match vector_type.element_type() {
                    "f32" => {
                        let float_type = self.context.f32_type();
                        Ok(float_type.const_float(*value).into())
                    }
                    "f64" => {
                        let float_type = self.context.f64_type();
                        Ok(float_type.const_float(*value).into())
                    }
                    _ => {
                        Err(CompileError::codegen_error(
                            format!("Cannot use float literal in {} vector", vector_type.element_type()),
                            None
                        ))
                    }
                }
            }
            Expr::Literal(Literal::Integer(value)) => {
                // Generate integer with the correct width for the vector type
                match vector_type.element_type() {
                    "i8" => {
                        let int_type = self.context.i8_type();
                        Ok(int_type.const_int(*value as u64, true).into())
                    }
                    "i16" => {
                        let int_type = self.context.i16_type();
                        Ok(int_type.const_int(*value as u64, true).into())
                    }
                    "i32" => {
                        let int_type = self.context.i32_type();
                        Ok(int_type.const_int(*value as u64, true).into())
                    }
                    "i64" => {
                        let int_type = self.context.i64_type();
                        Ok(int_type.const_int(*value as u64, true).into())
                    }
                    "u8" | "u16" | "u32" => {
                        // For unsigned types, use the corresponding signed type in LLVM
                        // The actual signed/unsigned interpretation happens at the operation level
                        match vector_type.element_type() {
                            "u8" => Ok(self.context.i8_type().const_int(*value as u64, false).into()),
                            "u16" => Ok(self.context.i16_type().const_int(*value as u64, false).into()),
                            "u32" => Ok(self.context.i32_type().const_int(*value as u64, false).into()),
                            _ => unreachable!()
                        }
                    }
                    _ => {
                        Err(CompileError::codegen_error(
                            format!("Cannot use integer literal in {} vector", vector_type.element_type()),
                            None
                        ))
                    }
                }
            }
            _ => {
                // For non-literal elements, generate normally and let type checking handle compatibility
                self.generate_expression(element)
            }
        }
    }
    
    /// Generates code for SIMD vector literal.
    fn generate_simd_vector_literal(
        &mut self, 
        elements: &[Expr], 
        vector_type: &SIMDVectorType
    ) -> Result<VectorValue<'ctx>> {
        // Generate values for each element with correct type for the vector
        let mut element_values = Vec::new();
        for element in elements {
            let element_value = self.generate_simd_element(element, vector_type)?;
            element_values.push(element_value);
        }
        
        // Validate element count matches vector width
        if element_values.len() != vector_type.width() {
            return Err(CompileError::codegen_error(
                format!(
                    "Vector literal element count mismatch: {} expected {}, got {}",
                    vector_type, vector_type.width(), element_values.len()
                ),
                None
            ));
        }
        
        // Create LLVM vector constant based on element type
        match vector_type.element_type() {
            "f32" | "f64" => {
                // For float vectors, create a constant vector
                if let Some(first_val) = element_values.first() {
                    if let BasicValueEnum::FloatValue(float_val) = first_val {
                        let const_vals: Vec<_> = element_values.iter()
                            .filter_map(|v| {
                                if let BasicValueEnum::FloatValue(fv) = v {
                                    Some(*fv)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        
                        if const_vals.len() == element_values.len() {
                            Ok(VectorType::const_vector(&const_vals))
                        } else {
                            Err(CompileError::codegen_error(
                                "Not all elements are float values".to_string(),
                                None
                            ))
                        }
                    } else {
                        Err(CompileError::codegen_error(
                            "Expected float values for float vector".to_string(),
                            None
                        ))
                    }
                } else {
                    Err(CompileError::codegen_error(
                        "Empty vector literal".to_string(),
                        None
                    ))
                }
            }
            _ => {
                // For integer and boolean vectors
                if let Some(first_val) = element_values.first() {
                    if let BasicValueEnum::IntValue(int_val) = first_val {
                        let const_vals: Vec<_> = element_values.iter()
                            .filter_map(|v| {
                                if let BasicValueEnum::IntValue(iv) = v {
                                    Some(*iv)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        
                        if const_vals.len() == element_values.len() {
                            Ok(VectorType::const_vector(&const_vals))
                        } else {
                            Err(CompileError::codegen_error(
                                "Not all elements are integer values".to_string(),
                                None
                            ))
                        }
                    } else {
                        Err(CompileError::codegen_error(
                            "Expected integer values for integer vector".to_string(),
                            None
                        ))
                    }
                } else {
                    Err(CompileError::codegen_error(
                        "Empty vector literal".to_string(),
                        None
                    ))
                }
            }
        }
    }
    
    /// Generates code for SIMD element-wise operations.
    fn generate_simd_elementwise(
        &mut self,
        left: &Expr,
        operator: &SIMDOperator,
        right: &Expr
    ) -> Result<BasicValueEnum<'ctx>> {
        let left_val = self.generate_expression(left)?;
        let right_val = self.generate_expression(right)?;
        
        // Extract vector values
        let (left_vec, right_vec) = match (left_val, right_val) {
            (BasicValueEnum::VectorValue(lv), BasicValueEnum::VectorValue(rv)) => (lv, rv),
            _ => return Err(CompileError::codegen_error(
                "Element-wise operations require vector operands".to_string(),
                None
            )),
        };
        
        // Generate the appropriate LLVM instruction based on operator
        let result = match operator {
            SIMDOperator::DotAdd => {
                // Check if we're dealing with float or integer vectors
                if left_vec.get_type().get_element_type().is_float_type() {
                    self.builder.build_float_add(left_vec, right_vec, "simd_fadd")
                        .map(|v| v.into())
                } else {
                    self.builder.build_int_add(left_vec, right_vec, "simd_add")
                        .map(|v| v.into())
                }
            }
            SIMDOperator::DotMultiply => {
                if left_vec.get_type().get_element_type().is_float_type() {
                    self.builder.build_float_mul(left_vec, right_vec, "simd_fmul")
                        .map(|v| v.into())
                } else {
                    self.builder.build_int_mul(left_vec, right_vec, "simd_mul")
                        .map(|v| v.into())
                }
            }
            SIMDOperator::DotDivide => {
                if left_vec.get_type().get_element_type().is_float_type() {
                    self.builder.build_float_div(left_vec, right_vec, "simd_fdiv")
                        .map(|v| v.into())
                } else {
                    // For integer division, use signed division
                    self.builder.build_int_signed_div(left_vec, right_vec, "simd_sdiv")
                        .map(|v| v.into())
                }
            }
            SIMDOperator::DotAnd => {
                self.builder.build_and(left_vec, right_vec, "simd_and")
                    .map(|v| v.into())
            }
            SIMDOperator::DotOr => {
                self.builder.build_or(left_vec, right_vec, "simd_or")
                    .map(|v| v.into())
            }
            SIMDOperator::DotXor => {
                self.builder.build_xor(left_vec, right_vec, "simd_xor")
                    .map(|v| v.into())
            }
        };
        
        result.map_err(|_| CompileError::codegen_error(
            format!("Failed to generate SIMD {:?} operation", operator),
            None
        ))
    }
    
    /// Generates code for a complete SIMD expression.
    fn generate_simd_expression(&mut self, simd_expr: &SIMDExpr) -> Result<BasicValueEnum<'ctx>> {
        match simd_expr {
            SIMDExpr::VectorLiteral { elements, vector_type, .. } => {
                if let Some(vtype) = vector_type {
                    let vector_val = self.generate_simd_vector_literal(elements, vtype)?;
                    Ok(vector_val.into())
                } else {
                    Err(CompileError::codegen_error(
                        "Vector literal must have explicit type annotation".to_string(),
                        None
                    ))
                }
            }
            SIMDExpr::ElementWise { left, operator, right, .. } => {
                self.generate_simd_elementwise(left, operator, right)
            }
            SIMDExpr::Broadcast { value, target_type, .. } => {
                // Generate scalar value
                let scalar_val = self.generate_expression(value)?;
                
                // Create vector by broadcasting scalar
                let vector_width = target_type.width();
                let vector_vals = vec![scalar_val; vector_width];
                
                // Create constant vector
                match scalar_val {
                    BasicValueEnum::FloatValue(fv) => {
                        let const_vals = vec![fv; vector_width];
                        Ok(VectorType::const_vector(&const_vals).into())
                    }
                    BasicValueEnum::IntValue(iv) => {
                        let const_vals = vec![iv; vector_width];
                        Ok(VectorType::const_vector(&const_vals).into())
                    }
                    _ => Err(CompileError::codegen_error(
                        "Unsupported scalar type for broadcast".to_string(),
                        None
                    ))
                }
            }
            SIMDExpr::Swizzle { vector, pattern, .. } => {
                self.generate_simd_swizzle(vector, pattern)
            }
            SIMDExpr::Reduction { vector, operation, .. } => {
                let vector_val = self.generate_expression(vector)?;
                
                if let BasicValueEnum::VectorValue(vec_val) = vector_val {
                    // For now, implement simple horizontal reduction by extracting and adding elements
                    // TODO: Use target-specific horizontal instructions when available
                    let vector_type = vec_val.get_type();
                    let element_count = vector_type.get_size();
                    
                    if element_count == 0 {
                        return Err(CompileError::codegen_error(
                            "Cannot reduce empty vector".to_string(),
                            None
                        ));
                    }
                    
                    // Extract first element as accumulator
                    let zero_index = self.context.i32_type().const_int(0, false);
                    let mut accumulator = self.builder.build_extract_element(vec_val, zero_index, "extract_0")
                        .map_err(|_| CompileError::codegen_error(
                            "Failed to extract vector element".to_string(),
                            None
                        ))?;
                    
                    // Reduce remaining elements
                    for i in 1..element_count {
                        let index = self.context.i32_type().const_int(i as u64, false);
                        let element = self.builder.build_extract_element(vec_val, index, &format!("extract_{}", i))
                            .map_err(|_| CompileError::codegen_error(
                                "Failed to extract vector element".to_string(),
                                None
                            ))?;
                        
                        accumulator = match operation {
                            crate::ast::ReductionOp::Sum => {
                                if vector_type.get_element_type().is_float_type() {
                                    if let (BasicValueEnum::FloatValue(acc), BasicValueEnum::FloatValue(elem)) = (accumulator, element) {
                                        self.builder.build_float_add(acc, elem, "reduce_add")
                                            .map(|v| v.into())
                                            .map_err(|_| CompileError::codegen_error("Failed to build float add".to_string(), None))?
                                    } else {
                                        return Err(CompileError::codegen_error("Type mismatch in reduction".to_string(), None));
                                    }
                                } else {
                                    if let (BasicValueEnum::IntValue(acc), BasicValueEnum::IntValue(elem)) = (accumulator, element) {
                                        self.builder.build_int_add(acc, elem, "reduce_add")
                                            .map(|v| v.into())
                                            .map_err(|_| CompileError::codegen_error("Failed to build int add".to_string(), None))?
                                    } else {
                                        return Err(CompileError::codegen_error("Type mismatch in reduction".to_string(), None));
                                    }
                                }
                            }
                            crate::ast::ReductionOp::Product => {
                                if vector_type.get_element_type().is_float_type() {
                                    if let (BasicValueEnum::FloatValue(acc), BasicValueEnum::FloatValue(elem)) = (accumulator, element) {
                                        self.builder.build_float_mul(acc, elem, "reduce_mul")
                                            .map(|v| v.into())
                                            .map_err(|_| CompileError::codegen_error("Failed to build float mul".to_string(), None))?
                                    } else {
                                        return Err(CompileError::codegen_error("Type mismatch in reduction".to_string(), None));
                                    }
                                } else {
                                    if let (BasicValueEnum::IntValue(acc), BasicValueEnum::IntValue(elem)) = (accumulator, element) {
                                        self.builder.build_int_mul(acc, elem, "reduce_mul")
                                            .map(|v| v.into())
                                            .map_err(|_| CompileError::codegen_error("Failed to build int mul".to_string(), None))?
                                    } else {
                                        return Err(CompileError::codegen_error("Type mismatch in reduction".to_string(), None));
                                    }
                                }
                            }
                            _ => {
                                return Err(CompileError::codegen_error(
                                    format!("Reduction operation {:?} not yet implemented", operation),
                                    None
                                ));
                            }
                        };
                    }
                    
                    Ok(accumulator)
                } else {
                    Err(CompileError::codegen_error(
                        "Reduction requires vector operand".to_string(),
                        None
                    ))
                }
            }
        }
    }
    
    /// Generates code for SIMD swizzle operations.
    fn generate_simd_swizzle(
        &mut self,
        vector: &Expr,
        pattern: &crate::ast::SwizzlePattern
    ) -> Result<BasicValueEnum<'ctx>> {
        let vector_val = self.generate_expression(vector)?;
        
        if let BasicValueEnum::VectorValue(vec_val) = vector_val {
            let vector_type = vec_val.get_type();
            let element_count = vector_type.get_size();
            
            // Convert swizzle pattern to indices
            let indices = self.swizzle_pattern_to_indices(pattern, element_count as usize)?;
            
            // Create shuffle mask
            let mask_values: Vec<_> = indices.iter()
                .map(|&i| self.context.i32_type().const_int(i as u64, false))
                .collect();
            
            // Create the shuffle mask vector
            let mask_vector = VectorType::const_vector(&mask_values);
            
            // Generate shuffle instruction
            self.builder.build_shuffle_vector(
                vec_val, 
                vec_val, // Use same vector for both operands
                mask_vector, 
                "swizzle"
            ).map(|v| v.into())
            .map_err(|_| CompileError::codegen_error(
                "Failed to generate swizzle operation".to_string(),
                None
            ))
        } else {
            Err(CompileError::codegen_error(
                "Swizzle requires vector operand".to_string(),
                None
            ))
        }
    }
    
    /// Converts swizzle pattern to element indices.
    fn swizzle_pattern_to_indices(&self, pattern: &crate::ast::SwizzlePattern, vector_width: usize) -> Result<Vec<u32>> {
        use crate::ast::SwizzlePattern;
        
        match pattern {
            SwizzlePattern::Named(name) => {
                let mut indices = Vec::new();
                for ch in name.chars() {
                    let index = match ch {
                        'x' => 0,
                        'y' => 1,
                        'z' => 2,
                        'w' => 3,
                        _ => return Err(CompileError::codegen_error(
                            format!("Invalid swizzle component: {}", ch),
                            None
                        ))
                    };
                    
                    if index >= vector_width as u32 {
                        return Err(CompileError::codegen_error(
                            format!("Swizzle index {} out of bounds for vector width {}", index, vector_width),
                            None
                        ));
                    }
                    
                    indices.push(index);
                }
                
                if indices.is_empty() {
                    return Err(CompileError::codegen_error(
                        "Empty named swizzle pattern".to_string(),
                        None
                    ));
                }
                
                Ok(indices)
            }
            SwizzlePattern::Range { start, end } => {
                if *start >= *end || *end > vector_width {
                    return Err(CompileError::codegen_error(
                        format!("Invalid range [{}:{}] for vector width {}", start, end, vector_width),
                        None
                    ));
                }
                
                let indices: Vec<u32> = (*start..*end).map(|i| i as u32).collect();
                Ok(indices)
            }
            SwizzlePattern::Indices(index_list) => {
                for &index in index_list {
                    if index >= vector_width {
                        return Err(CompileError::codegen_error(
                            format!("Swizzle index {} out of bounds for vector width {}", index, vector_width),
                            None
                        ));
                    }
                }
                
                if index_list.is_empty() {
                    return Err(CompileError::codegen_error(
                        "Empty index list in swizzle pattern".to_string(),
                        None
                    ));
                }
                
                let indices: Vec<u32> = index_list.iter().map(|&i| i as u32).collect();
                Ok(indices)
            }
        }
    }
    
    /// Generates optimized reduction operations using tree-based algorithms.
    fn generate_optimized_reduction(
        &mut self,
        vector: VectorValue<'ctx>,
        operation: &crate::ast::ReductionOp
    ) -> Result<BasicValueEnum<'ctx>> {
        let vector_type = vector.get_type();
        let element_count = vector_type.get_size();
        
        // Use tree-based reduction for larger vectors, linear for smaller ones
        if element_count > 4 {
            self.generate_tree_reduction(vector, operation)
        } else {
            self.generate_linear_reduction(vector, operation)
        }
    }
    
    /// Generates tree-based reduction for efficient parallel reduction.
    fn generate_tree_reduction(
        &mut self,
        vector: VectorValue<'ctx>,
        operation: &crate::ast::ReductionOp
    ) -> Result<BasicValueEnum<'ctx>> {
        let vector_type = vector.get_type();
        let element_count = vector_type.get_size();
        
        if element_count == 0 {
            return Err(CompileError::codegen_error(
                "Cannot reduce empty vector".to_string(),
                None
            ));
        }
        
        if element_count == 1 {
            // Single element - just extract it
            let zero_index = self.context.i32_type().const_int(0, false);
            return self.builder.build_extract_element(vector, zero_index, "single_element")
                .map_err(|_| CompileError::codegen_error(
                    "Failed to extract single element".to_string(),
                    None
                ));
        }
        
        // Tree reduction: repeatedly split vector in half and combine
        let mut current_vector = vector;
        let mut current_width = element_count;
        
        while current_width > 1 {
            let half_width = current_width / 2;
            let remaining = current_width % 2;
            
            // Create shuffle masks for the two halves
            let left_mask: Vec<_> = (0..half_width)
                .map(|i| self.context.i32_type().const_int(i as u64, false))
                .collect();
            let right_mask: Vec<_> = (half_width..current_width-remaining)
                .map(|i| self.context.i32_type().const_int(i as u64, false))
                .collect();
            
            if left_mask.len() == right_mask.len() && !left_mask.is_empty() {
                // Create vector halves
                let left_mask_vec = VectorType::const_vector(&left_mask);
                let right_mask_vec = VectorType::const_vector(&right_mask);
                
                let left_half = self.builder.build_shuffle_vector(
                    current_vector, current_vector, left_mask_vec, "left_half"
                ).map_err(|_| CompileError::codegen_error(
                    "Failed to create left half".to_string(), None
                ))?;
                
                let right_half = self.builder.build_shuffle_vector(
                    current_vector, current_vector, right_mask_vec, "right_half"
                ).map_err(|_| CompileError::codegen_error(
                    "Failed to create right half".to_string(), None
                ))?;
                
                // Combine the two halves
                current_vector = match operation {
                    crate::ast::ReductionOp::Sum => {
                        if vector_type.get_element_type().is_float_type() {
                            self.builder.build_float_add(left_half, right_half, "tree_add")
                        } else {
                            self.builder.build_int_add(left_half, right_half, "tree_add")
                        }
                    }
                    crate::ast::ReductionOp::Product => {
                        if vector_type.get_element_type().is_float_type() {
                            self.builder.build_float_mul(left_half, right_half, "tree_mul")
                        } else {
                            self.builder.build_int_mul(left_half, right_half, "tree_mul")
                        }
                    }
                    _ => return Err(CompileError::codegen_error(
                        format!("Tree reduction for {:?} not implemented", operation),
                        None
                    ))
                }.map_err(|_| CompileError::codegen_error(
                    "Failed to build tree reduction operation".to_string(),
                    None
                ))?;
                
                current_width = half_width;
            } else {
                // Fallback to linear reduction for odd sizes
                return self.generate_linear_reduction(current_vector, operation);
            }
        }
        
        // Extract the final result
        let zero_index = self.context.i32_type().const_int(0, false);
        self.builder.build_extract_element(current_vector, zero_index, "tree_result")
            .map_err(|_| CompileError::codegen_error(
                "Failed to extract tree reduction result".to_string(),
                None
            ))
    }
    
    /// Generates linear reduction for small vectors.
    fn generate_linear_reduction(
        &mut self,
        vector: VectorValue<'ctx>,
        operation: &crate::ast::ReductionOp
    ) -> Result<BasicValueEnum<'ctx>> {
        let vector_type = vector.get_type();
        let element_count = vector_type.get_size();
        
        if element_count == 0 {
            return Err(CompileError::codegen_error(
                "Cannot reduce empty vector".to_string(),
                None
            ));
        }
        
        // Extract first element as accumulator
        let zero_index = self.context.i32_type().const_int(0, false);
        let mut accumulator = self.builder.build_extract_element(vector, zero_index, "linear_acc_0")
            .map_err(|_| CompileError::codegen_error(
                "Failed to extract first element".to_string(),
                None
            ))?;
        
        // Linearly reduce remaining elements
        for i in 1..element_count {
            let index = self.context.i32_type().const_int(i as u64, false);
            let element = self.builder.build_extract_element(vector, index, &format!("linear_elem_{}", i))
                .map_err(|_| CompileError::codegen_error(
                    "Failed to extract vector element".to_string(),
                    None
                ))?;
            
            accumulator = match operation {
                crate::ast::ReductionOp::Sum => {
                    if vector_type.get_element_type().is_float_type() {
                        if let (BasicValueEnum::FloatValue(acc), BasicValueEnum::FloatValue(elem)) = (accumulator, element) {
                            self.builder.build_float_add(acc, elem, "linear_add")
                                .map(|v| v.into())
                                .map_err(|_| CompileError::codegen_error("Failed to build linear add".to_string(), None))?
                        } else {
                            return Err(CompileError::codegen_error("Type mismatch in linear reduction".to_string(), None));
                        }
                    } else {
                        if let (BasicValueEnum::IntValue(acc), BasicValueEnum::IntValue(elem)) = (accumulator, element) {
                            self.builder.build_int_add(acc, elem, "linear_add")
                                .map(|v| v.into())
                                .map_err(|_| CompileError::codegen_error("Failed to build linear add".to_string(), None))?
                        } else {
                            return Err(CompileError::codegen_error("Type mismatch in linear reduction".to_string(), None));
                        }
                    }
                }
                crate::ast::ReductionOp::Product => {
                    if vector_type.get_element_type().is_float_type() {
                        if let (BasicValueEnum::FloatValue(acc), BasicValueEnum::FloatValue(elem)) = (accumulator, element) {
                            self.builder.build_float_mul(acc, elem, "linear_mul")
                                .map(|v| v.into())
                                .map_err(|_| CompileError::codegen_error("Failed to build linear mul".to_string(), None))?
                        } else {
                            return Err(CompileError::codegen_error("Type mismatch in linear reduction".to_string(), None));
                        }
                    } else {
                        if let (BasicValueEnum::IntValue(acc), BasicValueEnum::IntValue(elem)) = (accumulator, element) {
                            self.builder.build_int_mul(acc, elem, "linear_mul")
                                .map(|v| v.into())
                                .map_err(|_| CompileError::codegen_error("Failed to build linear mul".to_string(), None))?
                        } else {
                            return Err(CompileError::codegen_error("Type mismatch in linear reduction".to_string(), None));
                        }
                    }
                }
                _ => {
                    return Err(CompileError::codegen_error(
                        format!("Linear reduction for {:?} not implemented", operation),
                        None
                    ));
                }
            };
        }
        
        Ok(accumulator)
    }
    
    /// Adds SIMD-specific performance optimization hints to a function.
    fn optimize_simd_function(&mut self, function: FunctionValue<'ctx>) -> Result<()> {
        // Add function-level optimizations for SIMD operations
        self.add_simd_function_attributes(function)?;
        self.optimize_simd_register_allocation(function)?;
        Ok(())
    }
    
    /// Adds SIMD-optimized function attributes.
    fn add_simd_function_attributes(&self, function: FunctionValue<'ctx>) -> Result<()> {
        // Add attributes to hint the optimizer about SIMD usage
        let context = function.get_type().get_context();
        
        // Prefer vector width that matches the most common SIMD operations
        let prefer_vector_width = context.create_string_attribute("prefer-vector-width", "256");
        function.add_attribute(inkwell::attributes::AttributeLoc::Function, prefer_vector_width);
        
        // Enable unsafe FP math for better SIMD optimization (when appropriate)
        let unsafe_fp_math = context.create_string_attribute("unsafe-fp-math", "true");
        function.add_attribute(inkwell::attributes::AttributeLoc::Function, unsafe_fp_math);
        
        // Target CPU features for better SIMD instruction selection
        let target_features = context.create_string_attribute("target-features", "+avx2,+fma");
        function.add_attribute(inkwell::attributes::AttributeLoc::Function, target_features);
        
        Ok(())
    }
    
    /// Optimizes register allocation for SIMD operations.
    fn optimize_simd_register_allocation(&self, function: FunctionValue<'ctx>) -> Result<()> {
        // Iterate through all basic blocks and instructions to add SIMD-specific hints
        for basic_block in function.get_basic_blocks() {
            for instruction in basic_block.get_instructions() {
                if self.is_simd_instruction(&instruction) {
                    // Add metadata to prefer vector registers
                    // Note: LLVM instruction-level optimization hints are typically
                    // applied through function attributes and optimization passes
                    
                    // Hint for vectorization
                    // Note: In LLVM IR, attributes are typically added at the function level
                    // Instruction-level optimization happens during the optimization passes
                }
            }
        }
        Ok(())
    }
    
    /// Checks if an instruction is a SIMD operation.
    fn is_simd_instruction(&self, instruction: &inkwell::values::InstructionValue) -> bool {
        // Check if instruction operates on vector types
        match instruction.get_opcode() {
            inkwell::values::InstructionOpcode::Add |
            inkwell::values::InstructionOpcode::FAdd |
            inkwell::values::InstructionOpcode::Sub |
            inkwell::values::InstructionOpcode::FSub |
            inkwell::values::InstructionOpcode::Mul |
            inkwell::values::InstructionOpcode::FMul |
            inkwell::values::InstructionOpcode::UDiv |
            inkwell::values::InstructionOpcode::SDiv |
            inkwell::values::InstructionOpcode::FDiv |
            inkwell::values::InstructionOpcode::And |
            inkwell::values::InstructionOpcode::Or |
            inkwell::values::InstructionOpcode::Xor => {
                // Check if operands are vector types
                if let Some(operand) = instruction.get_operand(0) {
                    if let Some(operand_value) = operand.left() {
                        return operand_value.get_type().is_vector_type();
                    }
                }
                false
            }
            inkwell::values::InstructionOpcode::ShuffleVector |
            inkwell::values::InstructionOpcode::ExtractElement |
            inkwell::values::InstructionOpcode::InsertElement => true,
            _ => false
        }
    }
    
    /// Enables auto-vectorization optimization for the module.
    pub fn enable_auto_vectorization(&mut self) -> Result<()> {
        // Add global metadata to enable LLVM's auto-vectorization
        let context = self.context;
        
        // Enable loop vectorization through module flags
        // Note: These are global hints that affect the LLVM optimizer behavior
        
        // In practice, vectorization is typically enabled via optimization passes
        // rather than global metadata. This is a conceptual implementation.
        
        // The actual auto-vectorization happens during LLVM optimization passes
        // which can be enabled when compiling the module to object code
        
        Ok(())
    }
    
    /// Generates optimized SIMD memory operations with alignment hints.
    fn generate_simd_load_aligned(
        &mut self,
        address: inkwell::values::PointerValue<'ctx>,
        vector_type: &SIMDVectorType
    ) -> Result<VectorValue<'ctx>> {
        let _llvm_type = self.simd_type_to_llvm(vector_type)?;
        
        // Create aligned load with appropriate alignment
        let alignment = self.get_simd_alignment(vector_type);
        let load_inst = self.builder.build_load(address, "simd_load_aligned")
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build aligned load: {:?}", e),
                None
            ))?;
        
        // Set alignment hint for better optimization
        if let Some(inst) = load_inst.as_instruction_value() {
            inst.set_alignment(alignment).map_err(|e| CompileError::codegen_error(
                format!("Failed to set load alignment: {:?}", e),
                None
            ))?;
        }
        
        // Convert to vector value
        if let inkwell::values::BasicValueEnum::VectorValue(vec_val) = load_inst {
            Ok(vec_val)
        } else {
            Err(CompileError::codegen_error(
                "Load result is not a vector value".to_string(),
                None
            ))
        }
    }
    
    /// Generates optimized SIMD memory store with alignment hints.
    fn generate_simd_store_aligned(
        &mut self,
        vector: VectorValue<'ctx>,
        address: inkwell::values::PointerValue<'ctx>,
        vector_type: &SIMDVectorType
    ) -> Result<()> {
        // Create aligned store with appropriate alignment
        let alignment = self.get_simd_alignment(vector_type);
        let store_inst = self.builder.build_store(address, vector)
            .map_err(|e| CompileError::codegen_error(
                format!("Failed to build aligned store: {:?}", e),
                None
            ))?;
        
        // Set alignment hint for better optimization
        store_inst.set_alignment(alignment).map_err(|e| CompileError::codegen_error(
            format!("Failed to set store alignment: {:?}", e),
            None
        ))?;
        
        Ok(())
    }
    
    /// Gets the appropriate memory alignment for a SIMD vector type.
    fn get_simd_alignment(&self, vector_type: &SIMDVectorType) -> u32 {
        match vector_type {
            // 128-bit vectors (16-byte alignment)
            SIMDVectorType::F32x4 | SIMDVectorType::F64x2 |
            SIMDVectorType::I32x4 | SIMDVectorType::I64x2 |
            SIMDVectorType::I16x8 | SIMDVectorType::I8x16 |
            SIMDVectorType::U32x4 | SIMDVectorType::U16x8 | SIMDVectorType::U8x16 => 16,
            
            // 256-bit vectors (32-byte alignment)
            SIMDVectorType::F32x8 | SIMDVectorType::F64x4 |
            SIMDVectorType::I32x8 | SIMDVectorType::I64x4 |
            SIMDVectorType::I16x16 | SIMDVectorType::I8x32 |
            SIMDVectorType::U32x8 | SIMDVectorType::U16x16 | SIMDVectorType::U8x32 => 32,
            
            // 512-bit vectors (64-byte alignment)
            SIMDVectorType::F32x16 | SIMDVectorType::F64x8 |
            SIMDVectorType::I32x16 | SIMDVectorType::I64x8 |
            SIMDVectorType::I16x32 | SIMDVectorType::I8x64 => 64,
            
            // Smaller vectors (8-byte alignment) 
            SIMDVectorType::F32x2 | SIMDVectorType::I32x2 |
            SIMDVectorType::I16x4 | SIMDVectorType::I8x8 => 8,
            
            // Mask types (match the underlying vector size they represent)
            SIMDVectorType::Mask8 => 8,
            SIMDVectorType::Mask16 => 16,
            SIMDVectorType::Mask32 => 32,
            SIMDVectorType::Mask64 => 64,
        }
    }
    
    /// Adds vectorization hints to loops containing SIMD operations.
    pub fn add_vectorization_hints(&mut self, _function: FunctionValue<'ctx>) -> Result<()> {
        // This would typically be called after generating a function with loops
        // Add metadata to encourage vectorization of loops containing SIMD operations
        
        let context = self.context;
        
        // Create vectorization metadata
        let _vectorize_enable = context.metadata_node(&[
            context.metadata_string("llvm.loop.vectorize.enable").into(),
            context.bool_type().const_int(1, false).into()
        ]);
        
        let _vectorize_width = context.metadata_node(&[
            context.metadata_string("llvm.loop.vectorize.width").into(),
            context.i32_type().const_int(8, false).into()
        ]);
        
        // Note: In a real implementation, these would be attached to specific loop blocks
        // This is a simplified version for demonstration
        
        Ok(())
    }
}