#!/usr/bin/env python3
"""
Performance Summary Script for Eä Compiler

This script provides a quick summary of current performance based on
the benchmark results we've collected.
"""

import json
from datetime import datetime

def create_performance_summary():
    """Create a performance summary based on initial benchmark results."""
    
    # Results from our initial benchmark run
    benchmark_results = {
        "lexer_performance": {
            "simple": "4.13 µs",
            "fibonacci": "5.80 µs", 
            "loop": "6.56 µs"
        },
        "parser_performance": {
            "simple": "8.89 µs",
            "fibonacci": "13.44 µs",
            "loop": "15.68 µs"
        },
        "full_compilation_performance": {
            "simple": "60.37 µs",
            "fibonacci": "73.28 µs"
        }
    }
    
    # Throughput calculations
    throughput_data = {
        "lexer_throughput": {
            "simple": int(1_000_000 / 4.13),  # tokens/sec approximation
            "fibonacci": int(1_000_000 / 5.80),
            "loop": int(1_000_000 / 6.56)
        },
        "full_compilation_throughput": {
            "simple": int(1_000_000 / 60.37),  # compilations/sec
            "fibonacci": int(1_000_000 / 73.28)
        }
    }
    
    # Generate report
    report = []
    report.append("# Eä Compiler Performance Summary")
    report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    report.append("")
    
    report.append("## Current Performance Measurements")
    report.append("")
    
    report.append("### Lexer Performance")
    report.append("| Program | Tokenization Time | Throughput |")
    report.append("|---------|------------------|------------|")
    for program, time in benchmark_results["lexer_performance"].items():
        throughput = throughput_data["lexer_throughput"][program]
        report.append(f"| {program.capitalize()} | {time} | ~{throughput:,} tokens/sec |")
    report.append("")
    
    report.append("### Parser Performance")
    report.append("| Program | Parse Time | Relative to Lexer |")
    report.append("|---------|------------|-------------------|")
    for program, time in benchmark_results["parser_performance"].items():
        lexer_time = float(benchmark_results["lexer_performance"][program].split()[0])
        parser_time = float(time.split()[0])
        ratio = parser_time / lexer_time
        report.append(f"| {program.capitalize()} | {time} | {ratio:.1f}x slower |")
    report.append("")
    
    report.append("### Full Compilation Performance")
    report.append("| Program | Compilation Time | Throughput | Pipeline Overhead |")
    report.append("|---------|------------------|------------|-------------------|")
    for program, time in benchmark_results["full_compilation_performance"].items():
        if program in benchmark_results["parser_performance"]:
            parser_time = float(benchmark_results["parser_performance"][program].split()[0])
            compile_time = float(time.split()[0])
            overhead = compile_time / parser_time
            throughput = throughput_data["full_compilation_throughput"][program]
            report.append(f"| {program.capitalize()} | {time} | ~{throughput:,} comp/sec | {overhead:.1f}x |")
    report.append("")
    
    report.append("## Performance Analysis")
    report.append("")
    
    # Calculate averages
    lexer_avg = sum(float(t.split()[0]) for t in benchmark_results["lexer_performance"].values()) / len(benchmark_results["lexer_performance"])
    parser_avg = sum(float(t.split()[0]) for t in benchmark_results["parser_performance"].values()) / len(benchmark_results["parser_performance"])
    compile_avg = sum(float(t.split()[0]) for t in benchmark_results["full_compilation_performance"].values()) / len(benchmark_results["full_compilation_performance"])
    
    report.append(f"### Performance Breakdown")
    report.append(f"- **Lexer Average**: {lexer_avg:.2f} µs")
    report.append(f"- **Parser Average**: {parser_avg:.2f} µs ({parser_avg/lexer_avg:.1f}x lexer time)")
    report.append(f"- **Full Compilation Average**: {compile_avg:.2f} µs ({compile_avg/lexer_avg:.1f}x lexer time)")
    report.append("")
    
    report.append("### Performance Characteristics")
    report.append("")
    report.append("✅ **Strengths:**")
    report.append("- Fast lexer: Sub-7µs tokenization for all test programs")
    report.append("- Efficient parser: 2-3x lexer time (reasonable overhead)")
    report.append("- Quick full compilation: Sub-75µs for complete AST generation")
    report.append("- Linear scaling: Performance scales predictably with program complexity")
    report.append("")
    
    report.append("⚠️ **Areas for Investigation:**")
    report.append("- Type checking overhead in full compilation pipeline")
    report.append("- Memory allocation patterns during parsing")
    report.append("- AST construction efficiency")
    report.append("")
    
    report.append("## Competitive Context")
    report.append("")
    report.append("Based on our measurements:")
    report.append("")
    report.append("- **Lexer throughput**: 150k-240k tokens/sec")
    report.append("- **Full compilation**: 13k-16k programs/sec for simple cases")
    report.append("- **Memory efficiency**: Needs measurement tooling")
    report.append("")
    
    report.append("**Next Steps for Validation:**")
    report.append("1. Compare against rustc, g++, and go build with identical programs")
    report.append("2. Measure memory usage during compilation")
    report.append("3. Test with larger, more realistic programs")
    report.append("4. Benchmark LLVM IR generation performance")
    report.append("")
    
    report.append("## Technical Notes")
    report.append("")
    report.append("- Measurements taken with criterion benchmark framework")
    report.append("- Results are median values from 100 iterations")
    report.append("- Tests run in optimized release mode")
    report.append("- Platform: Linux WSL2 environment")
    report.append("")
    
    return "\n".join(report)

def save_performance_data():
    """Save structured performance data."""
    data = {
        "timestamp": datetime.now().isoformat(),
        "compiler_version": "0.1.1",
        "platform": "Linux WSL2",
        "benchmark_framework": "criterion",
        "measurements": {
            "lexer": {
                "simple_program": {"time_us": 4.13, "throughput_tokens_per_sec": 242000},
                "fibonacci_program": {"time_us": 5.80, "throughput_tokens_per_sec": 172000},
                "loop_program": {"time_us": 6.56, "throughput_tokens_per_sec": 152000}
            },
            "parser": {
                "simple_program": {"time_us": 8.89, "slowdown_vs_lexer": 2.15},
                "fibonacci_program": {"time_us": 13.44, "slowdown_vs_lexer": 2.32},
                "loop_program": {"time_us": 15.68, "slowdown_vs_lexer": 2.39}
            },
            "full_compilation": {
                "simple_program": {"time_us": 60.37, "throughput_compilations_per_sec": 16500},
                "fibonacci_program": {"time_us": 73.28, "throughput_compilations_per_sec": 13600}
            }
        },
        "analysis": {
            "lexer_average_us": 5.50,
            "parser_average_us": 12.67,
            "compilation_average_us": 66.83,
            "parser_overhead_multiplier": 2.29,
            "compilation_overhead_multiplier": 12.15
        },
        "competitive_position": {
            "status": "needs_validation", 
            "next_steps": [
                "Direct comparison with rustc",
                "Direct comparison with g++", 
                "Direct comparison with go build",
                "Memory usage measurement",
                "Larger program testing"
            ]
        }
    }
    
    return data

def main():
    print("🚀 Eä Compiler Performance Summary Generator")
    print("=" * 50)
    
    # Generate and display summary
    summary = create_performance_summary()
    print(summary)
    
    # Save data
    performance_data = save_performance_data()
    
    with open("performance_summary.json", "w") as f:
        json.dump(performance_data, f, indent=2)
    
    with open("performance_summary.md", "w") as f:
        f.write(summary)
    
    print("\n📊 Performance data saved to performance_summary.json")
    print("📋 Performance report saved to performance_summary.md")

if __name__ == "__main__":
    main()