//! Language Server Protocol implementation for Eä
//!
//! Provides intelligent code completion, diagnostics, and performance analysis
//! tailored for the Eä programming language with special focus on SIMD optimizations.

#[cfg(feature = "lsp")]
use dashmap::DashMap;
#[cfg(feature = "lsp")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "lsp")]
use serde_json::Value;
#[cfg(feature = "lsp")]
use std::collections::HashMap;
#[cfg(feature = "lsp")]
use std::sync::Arc;
#[cfg(feature = "lsp")]
use tokio::sync::RwLock;
#[cfg(feature = "lsp")]
use tower_lsp::jsonrpc::Result;
#[cfg(feature = "lsp")]
use tower_lsp::lsp_types::*;
#[cfg(feature = "lsp")]
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[cfg(feature = "lsp")]
use crate::ast::Stmt;
#[cfg(feature = "lsp")]
use crate::lexer::{Lexer, Position as EaPosition, Token, TokenKind};
#[cfg(feature = "lsp")]
use crate::parser::Parser;
#[cfg(feature = "lsp")]
use crate::type_system::{TypeChecker, TypeContext};
#[cfg(feature = "lsp")]
use crate::{compile_to_ast, CompileError};

/// Performance analysis data for a function or expression
#[cfg(feature = "lsp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Estimated execution time in nanoseconds
    pub estimated_execution_time: u64,
    /// Memory usage in bytes
    pub estimated_memory_usage: usize,
    /// SIMD optimization opportunities
    pub simd_opportunities: Vec<SIMDOptimization>,
    /// Performance bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

#[cfg(feature = "lsp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIMDOptimization {
    /// Source range where SIMD can be applied
    pub range: Range,
    /// Type of SIMD operation suggested
    pub operation_type: String,
    /// Expected performance improvement
    pub performance_gain: f64,
    /// Human-readable description
    pub description: String,
}

#[cfg(feature = "lsp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Location of the bottleneck
    pub range: Range,
    /// Type of bottleneck (memory, cpu, io, etc.)
    pub bottleneck_type: String,
    /// Severity (1-10 scale)
    pub severity: u8,
    /// Description of the issue
    pub description: String,
}

#[cfg(feature = "lsp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Location to apply optimization
    pub range: Range,
    /// Type of optimization
    pub optimization_type: String,
    /// Expected improvement percentage
    pub expected_improvement: f64,
    /// Suggested code change
    pub suggestion: String,
    /// Rationale for the suggestion
    pub rationale: String,
}

/// Document state for incremental compilation
#[cfg(feature = "lsp")]
#[derive(Debug, Clone)]
struct DocumentState {
    /// Full text content
    content: String,
    /// Parsed AST
    ast: Option<Vec<Stmt>>,
    /// Type checking context
    type_context: Option<TypeContext>,
    /// Compilation errors
    errors: Vec<CompileError>,
    /// Performance analysis
    performance: Option<PerformanceAnalysis>,
    /// Document version for change tracking
    version: i32,
}

/// Main LSP server implementation
#[cfg(feature = "lsp")]
pub struct EaLanguageServer {
    /// LSP client handle
    client: Client,
    /// Document states indexed by URI
    documents: Arc<DashMap<String, DocumentState>>,
    /// Incremental compiler for fast re-analysis
    compiler_cache: Arc<RwLock<HashMap<String, CompilerCacheEntry>>>,
}

#[cfg(feature = "lsp")]
#[derive(Debug, Clone)]
struct CompilerCacheEntry {
    /// Source hash for cache validation
    source_hash: u64,
    /// Cached compilation result
    ast: Vec<Stmt>,
    /// Cached type context
    type_context: TypeContext,
    /// Performance analysis cache
    performance: PerformanceAnalysis,
}

#[cfg(feature = "lsp")]
impl EaLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            compiler_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze document for errors and performance
    async fn analyze_document(&self, uri: &str, content: &str, version: i32) -> DocumentState {
        let mut state = DocumentState {
            content: content.to_string(),
            ast: None,
            type_context: None,
            errors: Vec::new(),
            performance: None,
            version,
        };

        // Try to compile and analyze
        match compile_to_ast(content) {
            Ok((ast, type_context)) => {
                state.ast = Some(ast.clone());
                state.type_context = Some(type_context);

                // Perform performance analysis
                state.performance = Some(self.analyze_performance(&ast).await);
            }
            Err(error) => {
                state.errors.push(error);
            }
        }

        state
    }

    /// Analyze AST for performance characteristics
    async fn analyze_performance(&self, ast: &[Stmt]) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis {
            estimated_execution_time: 0,
            estimated_memory_usage: 0,
            simd_opportunities: Vec::new(),
            bottlenecks: Vec::new(),
            optimization_suggestions: Vec::new(),
        };

        // Simple performance heuristics for demonstration
        for stmt in ast {
            self.analyze_statement_performance(stmt, &mut analysis);
        }

        analysis
    }

    /// Analyze individual statement for performance
    fn analyze_statement_performance(&self, stmt: &Stmt, analysis: &mut PerformanceAnalysis) {
        match stmt {
            Stmt::FunctionDeclaration {
                name, params, body, ..
            } => {
                // Function calls have overhead
                analysis.estimated_execution_time += 5000; // 5μs function overhead
                analysis.estimated_memory_usage += 128 + (params.len() * 32); // Stack frame + parameters

                // Analyze function body
                if let Stmt::Block(statements) = body.as_ref() {
                    for stmt in statements {
                        self.analyze_statement_performance(stmt, analysis);
                    }
                }

                // Check for SIMD opportunities in mathematical functions
                if name.contains("calculate")
                    || name.contains("process")
                    || name.contains("transform")
                {
                    analysis.simd_opportunities.push(SIMDOptimization {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: name.len() as u32,
                            },
                        },
                        operation_type: "function_vectorization".to_string(),
                        performance_gain: 3.5,
                        description: format!(
                            "Function '{}' could benefit from SIMD vectorization",
                            name
                        ),
                    });
                }
            }

            Stmt::VarDeclaration { initializer, .. } => {
                analysis.estimated_execution_time += 500; // Variable allocation cost
                analysis.estimated_memory_usage += 32; // Base variable size

                if let Some(expr) = initializer {
                    self.analyze_expression_performance(expr, analysis);
                }
            }

            Stmt::Expression(expr) => {
                self.analyze_expression_performance(expr, analysis);
            }

            Stmt::Block(statements) => {
                // Block overhead for scope management
                analysis.estimated_execution_time += 200;
                analysis.estimated_memory_usage += 16;

                for stmt in statements {
                    self.analyze_statement_performance(stmt, analysis);
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Branch overhead
                analysis.estimated_execution_time += 1000;

                self.analyze_expression_performance(condition, analysis);
                self.analyze_statement_performance(then_branch, analysis);

                if let Some(else_stmt) = else_branch {
                    self.analyze_statement_performance(else_stmt, analysis);
                }

                // Suggest branch prediction optimization
                analysis
                    .optimization_suggestions
                    .push(OptimizationSuggestion {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 10,
                            },
                        },
                        optimization_type: "branch_optimization".to_string(),
                        expected_improvement: 15.0,
                        suggestion: "Consider using branchless operations for better performance"
                            .to_string(),
                        rationale:
                            "Branch misprediction can cause significant performance penalties"
                                .to_string(),
                    });
            }

            Stmt::Return(expr) => {
                analysis.estimated_execution_time += 300; // Return overhead

                if let Some(expr) = expr {
                    self.analyze_expression_performance(expr, analysis);
                }
            }

            _ => {
                // Default analysis for other statement types
                analysis.estimated_execution_time += 1000;
                analysis.estimated_memory_usage += 64;
            }
        }
    }

    /// Analyze expression for performance characteristics
    fn analyze_expression_performance(
        &self,
        expr: &crate::ast::Expr,
        analysis: &mut PerformanceAnalysis,
    ) {
        use crate::ast::Expr;

        match expr {
            Expr::Literal(_) => {
                // Literals are nearly free
                analysis.estimated_execution_time += 10;
            }

            Expr::Variable(_) => {
                // Variable access cost
                analysis.estimated_execution_time += 50;
            }

            Expr::Binary(left, op, right) => {
                // Binary operation cost
                analysis.estimated_execution_time += 200;

                // Recursively analyze operands
                self.analyze_expression_performance(left, analysis);
                self.analyze_expression_performance(right, analysis);

                // Check for SIMD optimization opportunities
                let op_str = format!("{:?}", op);
                if op_str.contains("Add")
                    || op_str.contains("Multiply")
                    || op_str.contains("Subtract")
                {
                    analysis.simd_opportunities.push(SIMDOptimization {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 10,
                            },
                        },
                        operation_type: format!("vectorized_{}", op_str.to_lowercase()),
                        performance_gain: 4.0,
                        description: format!("Binary {} operation can be vectorized", op_str),
                    });
                }
            }

            Expr::Call(func, args) => {
                // Function call overhead
                analysis.estimated_execution_time += 2000 + (args.len() as u64 * 100);

                // Analyze function arguments
                for arg in args {
                    self.analyze_expression_performance(arg, analysis);
                }

                // Check for mathematical functions that could use SIMD
                if let Expr::Variable(func_name) = func.as_ref() {
                    if ["sin", "cos", "sqrt", "abs", "max", "min"].contains(&func_name.as_str()) {
                        analysis.simd_opportunities.push(SIMDOptimization {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: func_name.len() as u32,
                                },
                            },
                            operation_type: "math_function_vectorization".to_string(),
                            performance_gain: 8.0,
                            description: format!(
                                "Math function '{}' can be vectorized for significant speedup",
                                func_name
                            ),
                        });
                    }
                }
            }

            Expr::Index(array, index) => {
                analysis.estimated_execution_time += 300; // Array access cost

                self.analyze_expression_performance(array, analysis);
                self.analyze_expression_performance(index, analysis);

                // Suggest potential cache optimization
                analysis
                    .optimization_suggestions
                    .push(OptimizationSuggestion {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 10,
                            },
                        },
                        optimization_type: "memory_access_pattern".to_string(),
                        expected_improvement: 25.0,
                        suggestion:
                            "Consider sequential access patterns for better cache performance"
                                .to_string(),
                        rationale: "Random memory access can cause cache misses".to_string(),
                    });
            }

            _ => {
                // Default cost for other expressions
                analysis.estimated_execution_time += 500;
            }
        }
    }

    /// Convert Eä position to LSP position
    fn ea_position_to_lsp(pos: &EaPosition) -> Position {
        Position {
            line: (pos.line - 1) as u32, // LSP is 0-indexed
            character: (pos.column - 1) as u32,
        }
    }

    /// Convert compile error to LSP diagnostic
    fn compile_error_to_diagnostic(error: &CompileError) -> Diagnostic {
        let (start_pos, end_pos) = match error {
            CompileError::LexError { position, .. } => {
                let pos = Self::ea_position_to_lsp(position);
                (
                    pos,
                    Position {
                        line: pos.line,
                        character: pos.character + 1,
                    },
                )
            }
            CompileError::ParseError { position, .. } => {
                let pos = Self::ea_position_to_lsp(position);
                (
                    pos,
                    Position {
                        line: pos.line,
                        character: pos.character + 5,
                    },
                )
            }
            CompileError::TypeError { position, .. } => {
                let pos = Self::ea_position_to_lsp(position);
                (
                    pos,
                    Position {
                        line: pos.line,
                        character: pos.character + 3,
                    },
                )
            }
            CompileError::CodeGenError { position, .. } => {
                if let Some(pos) = position {
                    let lsp_pos = Self::ea_position_to_lsp(pos);
                    (
                        lsp_pos,
                        Position {
                            line: lsp_pos.line,
                            character: lsp_pos.character + 5,
                        },
                    )
                } else {
                    // CodeGenError without position info, use document start
                    let pos = Position {
                        line: 0,
                        character: 0,
                    };
                    (
                        pos,
                        Position {
                            line: 0,
                            character: 10,
                        },
                    )
                }
            }
            CompileError::MemoryExhausted { .. } => {
                // MemoryExhausted without position info, use document start
                let pos = Position {
                    line: 0,
                    character: 0,
                };
                (
                    pos,
                    Position {
                        line: 0,
                        character: 15,
                    },
                )
            }
        };

        Diagnostic {
            range: Range {
                start: start_pos,
                end: end_pos,
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("ea-compiler".to_string()),
            message: error.to_string(),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Generate completion items for current context
    fn generate_completions(
        &self,
        position: Position,
        document_state: Option<&DocumentState>,
    ) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add basic language keywords
        completions.extend(self.get_keyword_completions());

        // Add SIMD vector types
        completions.extend(self.get_simd_type_completions());

        // Add built-in functions
        completions.extend(self.get_builtin_function_completions());

        // Add context-specific completions based on document state
        if let Some(state) = document_state {
            if let Some(ref ast) = state.ast {
                completions.extend(self.get_context_completions(ast, position));
            }
        }

        completions
    }

    /// Get language keyword completions
    fn get_keyword_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "func".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function declaration".to_string()),
                documentation: Some(Documentation::String(
                    "Declare a new function with parameters and return type".to_string(),
                )),
                insert_text: Some(
                    "func ${1:name}(${2:params}) -> ${3:type} {\n    ${4:body}\n}".to_string(),
                ),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Variable declaration".to_string()),
                documentation: Some(Documentation::String(
                    "Declare a new variable with type inference".to_string(),
                )),
                insert_text: Some("let ${1:name} = ${2:value};".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                documentation: Some(Documentation::String(
                    "Execute code conditionally based on a boolean expression".to_string(),
                )),
                insert_text: Some("if ${1:condition} {\n    ${2:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "while".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Loop statement".to_string()),
                documentation: Some(Documentation::String(
                    "Repeat code while a condition is true".to_string(),
                )),
                insert_text: Some("while ${1:condition} {\n    ${2:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "for".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("For loop statement".to_string()),
                documentation: Some(Documentation::String(
                    "Iterate over a range or collection".to_string(),
                )),
                insert_text: Some("for ${1:item} in ${2:iterator} {\n    ${3:body}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "return".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Return statement".to_string()),
                documentation: Some(Documentation::String(
                    "Return a value from a function".to_string(),
                )),
                insert_text: Some("return ${1:value};".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get SIMD vector type completions
    fn get_simd_type_completions(&self) -> Vec<CompletionItem> {
        let simd_types = [
            (
                "f32x4",
                "4-element single precision floating point SIMD vector",
            ),
            (
                "f32x8",
                "8-element single precision floating point SIMD vector",
            ),
            (
                "f64x2",
                "2-element double precision floating point SIMD vector",
            ),
            (
                "f64x4",
                "4-element double precision floating point SIMD vector",
            ),
            ("i8x16", "16-element 8-bit integer SIMD vector"),
            ("i16x8", "8-element 16-bit integer SIMD vector"),
            ("i32x4", "4-element 32-bit integer SIMD vector"),
            ("i32x8", "8-element 32-bit integer SIMD vector"),
            ("i64x2", "2-element 64-bit integer SIMD vector"),
            ("i64x4", "4-element 64-bit integer SIMD vector"),
            ("u8x16", "16-element unsigned 8-bit integer SIMD vector"),
            ("u16x8", "8-element unsigned 16-bit integer SIMD vector"),
            ("u32x4", "4-element unsigned 32-bit integer SIMD vector"),
            ("u32x8", "8-element unsigned 32-bit integer SIMD vector"),
            ("u64x2", "2-element unsigned 64-bit integer SIMD vector"),
            ("u64x4", "4-element unsigned 64-bit integer SIMD vector"),
        ];

        simd_types
            .iter()
            .map(|(name, description)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some("SIMD vector type".to_string()),
                documentation: Some(Documentation::String(description.to_string())),
                insert_text: Some(name.to_string()),
                ..Default::default()
            })
            .collect()
    }

    /// Get built-in function completions
    fn get_builtin_function_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "println".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn println(message: string) -> void".to_string()),
                documentation: Some(Documentation::String(
                    "Print a line to standard output".to_string(),
                )),
                insert_text: Some("println(${1:message})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "print".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn print(message: string) -> void".to_string()),
                documentation: Some(Documentation::String(
                    "Print to standard output without newline".to_string(),
                )),
                insert_text: Some("print(${1:message})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "sin".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn sin(x: f64) -> f64".to_string()),
                documentation: Some(Documentation::String(
                    "Calculate sine of x (SIMD optimized when used with vectors)".to_string(),
                )),
                insert_text: Some("sin(${1:x})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "cos".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn cos(x: f64) -> f64".to_string()),
                documentation: Some(Documentation::String(
                    "Calculate cosine of x (SIMD optimized when used with vectors)".to_string(),
                )),
                insert_text: Some("cos(${1:x})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "sqrt".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn sqrt(x: f64) -> f64".to_string()),
                documentation: Some(Documentation::String(
                    "Calculate square root of x (SIMD optimized when used with vectors)"
                        .to_string(),
                )),
                insert_text: Some("sqrt(${1:x})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "abs".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn abs(x: number) -> number".to_string()),
                documentation: Some(Documentation::String(
                    "Calculate absolute value of x (SIMD optimized when used with vectors)"
                        .to_string(),
                )),
                insert_text: Some("abs(${1:x})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "load_vector".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn load_vector<T>(ptr: *const T) -> vector<T>".to_string()),
                documentation: Some(Documentation::String(
                    "Load a SIMD vector from memory with proper alignment".to_string(),
                )),
                insert_text: Some("load_vector(${1:ptr})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "store_vector".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn store_vector<T>(ptr: *mut T, vec: vector<T>) -> void".to_string()),
                documentation: Some(Documentation::String(
                    "Store a SIMD vector to memory with proper alignment".to_string(),
                )),
                insert_text: Some("store_vector(${1:ptr}, ${2:vector})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get context-specific completions based on AST analysis
    fn get_context_completions(&self, ast: &[Stmt], _position: Position) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Extract user-defined functions and variables from AST
        for stmt in ast {
            self.extract_symbols_from_statement(stmt, &mut completions);
        }

        completions
    }

    /// Extract symbol completions from AST statements
    fn extract_symbols_from_statement(&self, stmt: &Stmt, completions: &mut Vec<CompletionItem>) {
        match stmt {
            Stmt::FunctionDeclaration {
                name,
                params,
                return_type,
                ..
            } => {
                let param_list: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, format!("{:?}", p.type_annotation)))
                    .collect();
                let param_signature = param_list.join(", ");
                let return_sig = return_type
                    .as_ref()
                    .map(|rt| format!(" -> {:?}", rt))
                    .unwrap_or_default();

                completions.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(format!("fn {}({}){}", name, param_signature, return_sig)),
                    documentation: Some(Documentation::String(format!(
                        "User-defined function '{}'",
                        name
                    ))),
                    insert_text: Some(format!(
                        "{}({})",
                        name,
                        params
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format!("${{{}:arg{}}}", i + 1, i + 1))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }

            Stmt::VarDeclaration {
                name,
                type_annotation,
                ..
            } => {
                let type_info = type_annotation
                    .as_ref()
                    .map(|ta| format!("{:?}", ta))
                    .unwrap_or_else(|| "inferred".to_string());

                completions.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some(format!("var {}: {}", name, type_info)),
                    documentation: Some(Documentation::String(format!("Variable '{}'", name))),
                    insert_text: Some(name.clone()),
                    ..Default::default()
                });
            }

            Stmt::Block(statements) => {
                for stmt in statements {
                    self.extract_symbols_from_statement(stmt, completions);
                }
            }

            _ => {} // Other statement types don't contribute to symbol completions
        }
    }
}

#[cfg(feature = "lsp")]
#[tower_lsp::async_trait]
impl LanguageServer for EaLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ea-compiler".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ea-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Eä Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let content = params.text_document.text;
        let version = params.text_document.version;

        let state = self.analyze_document(&uri, &content, version).await;

        // Send diagnostics
        let diagnostics: Vec<Diagnostic> = state
            .errors
            .iter()
            .map(Self::compile_error_to_diagnostic)
            .collect();

        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, Some(version))
            .await;

        // Send performance information as info message
        if let Some(perf) = &state.performance {
            let message = format!(
                "Performance Analysis: ~{}μs execution, {}KB memory, {} SIMD opportunities",
                perf.estimated_execution_time / 1000,
                perf.estimated_memory_usage / 1024,
                perf.simd_opportunities.len()
            );
            self.client.log_message(MessageType::INFO, message).await;
        }

        self.documents.insert(uri, state);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;
            let state = self.analyze_document(&uri, &content, version).await;

            // Send updated diagnostics
            let diagnostics: Vec<Diagnostic> = state
                .errors
                .iter()
                .map(Self::compile_error_to_diagnostic)
                .collect();

            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, Some(version))
                .await;

            self.documents.insert(uri, state);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let document_state = self.documents.get(&uri);
        let completions = self.generate_completions(position, document_state.as_deref());
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();

        if let Some(state) = self.documents.get(&uri) {
            if let Some(perf) = &state.performance {
                let hover_content = format!(
                    "**Eä Performance Analysis**\n\n\
                    • Estimated execution time: {}μs\n\
                    • Memory usage: {}KB\n\
                    • SIMD opportunities: {}\n\
                    • Optimization suggestions: {}",
                    perf.estimated_execution_time / 1000,
                    perf.estimated_memory_usage / 1024,
                    perf.simd_opportunities.len(),
                    perf.optimization_suggestions.len()
                );

                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_content,
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }
}

/// Create and run the LSP server
#[cfg(feature = "lsp")]
pub async fn run_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| EaLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
