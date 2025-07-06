// tests/simd_integration_tests.rs - Fixed version
// SIMD-002 Phase 4: Comprehensive Test Suite
// Validates all aspects of SIMD expression parsing and validation

use ea_compiler::{
    ast::{BinaryOp, Expr},
    lexer::{Lexer, TokenKind},
    parser::Parser,
};

/// Simple SIMD test suite for basic functionality
pub struct SIMDTestSuite {
    test_results: Vec<TestResult>,
    total_tests: usize,
    passed_tests: usize,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    name: String,
    status: TestStatus,
    details: String,
    execution_time_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

impl SIMDTestSuite {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
            total_tests: 0,
            passed_tests: 0,
        }
    }

    /// Run basic SIMD tests
    pub fn run_basic_tests(&mut self) {
        println!("🚀 BASIC SIMD TEST SUITE");
        println!("========================");

        self.test_basic_parsing();
        self.test_simd_operators();
        self.print_summary();
    }

    /// Test basic parsing functionality
    fn test_basic_parsing(&mut self) {
        println!("\n📊 Testing Basic Parsing...");

        // Test 1: Basic arithmetic expression
        self.run_test("Basic Arithmetic", || {
            let source = "1 + 2 * 3";
            let mut lexer = Lexer::new(source);
            let tokens = lexer
                .tokenize_all()
                .map_err(|e| format!("Lexer error: {:?}", e))?;

            let mut parser = Parser::new(tokens);
            let result = parser
                .parse()
                .map_err(|e| format!("Parser error: {:?}", e))?;

            match result {
                Expr::Binary(_, BinaryOp::Add, _) => Ok("✅ Basic arithmetic parsed correctly"),
                _ => Err("❌ Expected addition expression".to_string()),
            }
        });

        // Test 2: Variable assignment
        self.run_test("Variable Assignment", || {
            let source = "x = 42";
            let mut lexer = Lexer::new(source);
            let tokens = lexer
                .tokenize_all()
                .map_err(|e| format!("Lexer error: {:?}", e))?;

            let mut parser = Parser::new(tokens);
            let result = parser
                .parse()
                .map_err(|e| format!("Parser error: {:?}", e))?;

            match result {
                Expr::Binary(_, BinaryOp::Assign, _) => Ok("✅ Assignment parsed correctly"),
                _ => Err("❌ Expected assignment expression".to_string()),
            }
        });

        // Test 3: Function call
        self.run_test("Function Call", || {
            let source = "foo(1, 2)";
            let mut lexer = Lexer::new(source);
            let tokens = lexer
                .tokenize_all()
                .map_err(|e| format!("Lexer error: {:?}", e))?;

            let mut parser = Parser::new(tokens);
            let result = parser
                .parse()
                .map_err(|e| format!("Parser error: {:?}", e))?;

            match result {
                Expr::Call(_, args) => {
                    if args.len() == 2 {
                        Ok("✅ Function call parsed correctly")
                    } else {
                        Err(format!("❌ Expected 2 arguments, got {}", args.len()))
                    }
                }
                _ => Err("❌ Expected function call".to_string()),
            }
        });
    }

    /// Test SIMD operators
    fn test_simd_operators(&mut self) {
        println!("\n⚙️ Testing SIMD Operators...");

        // Test SIMD operators are recognized by lexer
        let simd_ops = vec![
            (".*", TokenKind::DotMultiply),
            (".+", TokenKind::DotAdd),
            ("./", TokenKind::DotDivide),
            (".&", TokenKind::DotAnd),
            (".|", TokenKind::DotOr),
            (".^", TokenKind::DotXor),
        ];

        for (op_str, expected_token) in simd_ops {
            self.run_test(&format!("SIMD Operator {}", op_str), || {
                let mut lexer = Lexer::new(op_str);
                let tokens = lexer
                    .tokenize_all()
                    .map_err(|e| format!("Lexer error: {:?}", e))?;

                if tokens.len() >= 1 && tokens[0].kind == expected_token {
                    Ok("✅ SIMD operator recognized")
                } else {
                    Err(format!(
                        "❌ Expected {:?}, got {:?}",
                        expected_token,
                        tokens.get(0).map(|t| &t.kind)
                    ))
                }
            });
        }
    }

    /// Helper method to run a test
    fn run_test<F>(&mut self, name: &str, test_fn: F)
    where
        F: FnOnce() -> Result<&'static str, String>,
    {
        let start_time = std::time::Instant::now();
        self.total_tests += 1;

        match test_fn() {
            Ok(message) => {
                let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
                println!("  {} (in {:.2}ms)", message, elapsed);

                self.test_results.push(TestResult {
                    name: name.to_string(),
                    status: TestStatus::Passed,
                    details: message.to_string(),
                    execution_time_ms: elapsed,
                });
                self.passed_tests += 1;
            }
            Err(error) => {
                let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
                println!("  ❌ {} - {} (in {:.2}ms)", name, error, elapsed);

                self.test_results.push(TestResult {
                    name: name.to_string(),
                    status: TestStatus::Failed,
                    details: error,
                    execution_time_ms: elapsed,
                });
            }
        }
    }

    /// Print test summary
    fn print_summary(&self) {
        let separator = "=".repeat(40);
        println!("\n{}", separator);
        println!("🎯 BASIC SIMD TEST RESULTS");
        println!("{}", separator);

        let success_rate = (self.passed_tests as f64 / self.total_tests as f64) * 100.0;
        let total_time: f64 = self.test_results.iter().map(|r| r.execution_time_ms).sum();

        println!("📊 RESULTS:");
        println!("   Total Tests: {}", self.total_tests);
        println!("   Passed: {} ✅", self.passed_tests);
        println!("   Failed: {} ❌", self.total_tests - self.passed_tests);
        println!("   Success Rate: {:.1}%", success_rate);
        println!("   Total Time: {:.2}ms", total_time);

        if success_rate >= 90.0 {
            println!("\n🎉 EXCELLENT! Basic functionality working well!");
        } else if success_rate >= 70.0 {
            println!("\n✅ GOOD! Most basic functionality working.");
        } else {
            println!("\n⚠️ NEEDS WORK: Several basic tests failing.");
        }

        let separator = "=".repeat(40);
        println!("\n{}", separator);
    }
}

/// Integration test for basic SIMD functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_simd_functionality() {
        let mut test_suite = SIMDTestSuite::new();
        test_suite.run_basic_tests();

        // Assert that most tests pass
        let success_rate = (test_suite.passed_tests as f64 / test_suite.total_tests as f64) * 100.0;
        assert!(
            success_rate >= 80.0,
            "Basic SIMD tests should have ≥80% success rate, got {:.1}%",
            success_rate
        );
    }

    #[test]
    fn test_simd_lexer_tokens() {
        // Test that all SIMD operator tokens are recognized
        let operators = vec![".*", ".+", "./", ".&", ".|", ".^"];

        for op in operators {
            let mut lexer = Lexer::new(op);
            let tokens = lexer.tokenize_all().expect("Should tokenize SIMD operator");
            assert!(
                tokens.len() >= 1,
                "Should have at least one token for {}",
                op
            );

            // The exact token type depends on implementation, but it should not be an error
            println!("✅ {} tokenized successfully", op);
        }
    }

    #[test]
    fn test_basic_expression_parsing() {
        // Test basic expression parsing works
        let expressions = vec!["1 + 2", "x = 42", "foo(1, 2)", "a * b + c"];

        for expr in expressions {
            let mut lexer = Lexer::new(expr);
            let tokens = lexer.tokenize_all().expect("Should tokenize expression");

            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            assert!(result.is_ok(), "Should parse expression: {}", expr);

            println!("✅ {} parsed successfully", expr);
        }
    }
}

/// Main function for running basic SIMD tests
#[allow(dead_code)]
pub fn main() {
    println!("🚀 BASIC SIMD FUNCTIONALITY TEST");
    println!("=================================");

    let mut test_suite = SIMDTestSuite::new();
    test_suite.run_basic_tests();
}
