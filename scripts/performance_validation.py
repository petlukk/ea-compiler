#!/usr/bin/env python3
"""
Evidence-Based Performance Validation Script for Eä Compiler

This script runs comprehensive benchmarks comparing Eä against
Rust, C++, and Go compilers to generate evidence-based performance reports.
"""

import subprocess
import time
import json
import sys
import os
import tempfile
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Optional, Tuple
import statistics

@dataclass
class CompilerInfo:
    name: str
    version: str
    available: bool

@dataclass
class BenchmarkResult:
    compiler: str
    test_name: str
    compilation_time_ms: float
    memory_usage_mb: Optional[float]
    success: bool
    output_size_bytes: Optional[int]

class PerformanceValidator:
    def __init__(self):
        self.results: List[BenchmarkResult] = []
        self.compilers = self.detect_compilers()
        
    def detect_compilers(self) -> Dict[str, CompilerInfo]:
        """Detect which compilers are available on the system."""
        compilers = {}
        
        # Check Eä compiler
        try:
            result = subprocess.run(['cargo', 'build', '--release', '--features=llvm'], 
                                  capture_output=True, text=True, cwd=Path(__file__).parent.parent)
            ea_available = result.returncode == 0
            compilers['ea'] = CompilerInfo('Eä', '0.1.1', ea_available)
        except FileNotFoundError:
            compilers['ea'] = CompilerInfo('Eä', '0.1.1', False)
        
        # Check Rust compiler
        try:
            result = subprocess.run(['rustc', '--version'], capture_output=True, text=True)
            if result.returncode == 0:
                version = result.stdout.strip().split()[1]
                compilers['rust'] = CompilerInfo('Rust', version, True)
            else:
                compilers['rust'] = CompilerInfo('Rust', 'unknown', False)
        except FileNotFoundError:
            compilers['rust'] = CompilerInfo('Rust', 'unknown', False)
        
        # Check C++ compiler
        try:
            result = subprocess.run(['g++', '--version'], capture_output=True, text=True)
            if result.returncode == 0:
                version = result.stdout.split('\n')[0]
                compilers['cpp'] = CompilerInfo('C++', version, True)
            else:
                compilers['cpp'] = CompilerInfo('C++', 'unknown', False)
        except FileNotFoundError:
            compilers['cpp'] = CompilerInfo('C++', 'unknown', False)
        
        # Check Go compiler
        try:
            result = subprocess.run(['go', 'version'], capture_output=True, text=True)
            if result.returncode == 0:
                version = result.stdout.strip().split()[2]
                compilers['go'] = CompilerInfo('Go', version, True)
            else:
                compilers['go'] = CompilerInfo('Go', 'unknown', False)
        except FileNotFoundError:
            compilers['go'] = CompilerInfo('Go', 'unknown', False)
        
        return compilers
    
    def create_test_programs(self) -> Dict[str, Dict[str, str]]:
        """Create equivalent test programs for all compilers."""
        return {
            'fibonacci': {
                'ea': '''
func fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() -> i32 {
    return fibonacci(20);
}
''',
                'rust': '''
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    println!("{}", fibonacci(20));
}
''',
                'cpp': '''
#include <iostream>

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    std::cout << fibonacci(20) << std::endl;
    return 0;
}
''',
                'go': '''
package main

import "fmt"

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
    fmt.Println(fibonacci(20))
}
'''
            },
            'sorting': {
                'ea': '''
func bubble_sort(n: i32) -> i32 {
    let swapped = true;
    let passes = 0;
    while swapped {
        swapped = false;
        let i = 1;
        while i < n {
            if i > i - 1 {
                swapped = true;
            }
            i = i + 1;
        }
        passes = passes + 1;
        n = n - 1;
    }
    return passes;
}

func main() -> i32 {
    return bubble_sort(100);
}
''',
                'rust': '''
fn bubble_sort(mut n: i32) -> i32 {
    let mut swapped = true;
    let mut passes = 0;
    while swapped {
        swapped = false;
        for i in 1..n {
            if i > i - 1 {  // Simplified comparison
                swapped = true;
            }
        }
        passes += 1;
        n -= 1;
    }
    passes
}

fn main() {
    println!("{}", bubble_sort(100));
}
''',
                'cpp': '''
#include <iostream>

int bubble_sort(int n) {
    bool swapped = true;
    int passes = 0;
    while (swapped) {
        swapped = false;
        for (int i = 1; i < n; ++i) {
            if (i > i - 1) {  // Simplified comparison
                swapped = true;
            }
        }
        passes++;
        n--;
    }
    return passes;
}

int main() {
    std::cout << bubble_sort(100) << std::endl;
    return 0;
}
''',
                'go': '''
package main

import "fmt"

func bubbleSort(n int) int {
    swapped := true
    passes := 0
    for swapped {
        swapped = false
        for i := 1; i < n; i++ {
            if i > i-1 {  // Simplified comparison
                swapped = true
            }
        }
        passes++
        n--
    }
    return passes
}

func main() {
    fmt.Println(bubbleSort(100))
}
'''
            }
        }
    
    def benchmark_ea_compiler(self, program: str, test_name: str) -> BenchmarkResult:
        """Benchmark the Eä compiler."""
        if not self.compilers['ea'].available:
            return BenchmarkResult('ea', test_name, 0, None, False, None)
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.ea', delete=False) as f:
            f.write(program)
            temp_file = f.name
        
        try:
            # Build the Eä compiler
            compiler_path = Path(__file__).parent.parent / 'target' / 'release' / 'ea'
            
            start_time = time.time()
            result = subprocess.run(
                [str(compiler_path), '--emit-llvm', temp_file],
                capture_output=True,
                text=True
            )
            compilation_time = (time.time() - start_time) * 1000  # Convert to milliseconds
            
            success = result.returncode == 0
            output_size = None
            
            # Check if LLVM file was created
            llvm_file = temp_file.replace('.ea', '.ll')
            if os.path.exists(llvm_file):
                output_size = os.path.getsize(llvm_file)
                os.unlink(llvm_file)
            
            return BenchmarkResult('ea', test_name, compilation_time, None, success, output_size)
            
        finally:
            os.unlink(temp_file)
    
    def benchmark_rust_compiler(self, program: str, test_name: str) -> BenchmarkResult:
        """Benchmark the Rust compiler."""
        if not self.compilers['rust'].available:
            return BenchmarkResult('rust', test_name, 0, None, False, None)
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.rs', delete=False) as f:
            f.write(program)
            temp_file = f.name
        
        try:
            start_time = time.time()
            result = subprocess.run(
                ['rustc', '-O', '--emit=llvm-ir', '-o', temp_file.replace('.rs', '.ll'), temp_file],
                capture_output=True,
                text=True
            )
            compilation_time = (time.time() - start_time) * 1000
            
            success = result.returncode == 0
            output_size = None
            
            llvm_file = temp_file.replace('.rs', '.ll')
            if os.path.exists(llvm_file):
                output_size = os.path.getsize(llvm_file)
                os.unlink(llvm_file)
            
            return BenchmarkResult('rust', test_name, compilation_time, None, success, output_size)
            
        finally:
            os.unlink(temp_file)
    
    def benchmark_cpp_compiler(self, program: str, test_name: str) -> BenchmarkResult:
        """Benchmark the C++ compiler."""
        if not self.compilers['cpp'].available:
            return BenchmarkResult('cpp', test_name, 0, None, False, None)
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.cpp', delete=False) as f:
            f.write(program)
            temp_file = f.name
        
        try:
            start_time = time.time()
            result = subprocess.run(
                ['g++', '-O2', '-S', '-emit-llvm', '-o', temp_file.replace('.cpp', '.ll'), temp_file],
                capture_output=True,
                text=True
            )
            compilation_time = (time.time() - start_time) * 1000
            
            success = result.returncode == 0
            output_size = None
            
            llvm_file = temp_file.replace('.cpp', '.ll')
            if os.path.exists(llvm_file):
                output_size = os.path.getsize(llvm_file)
                os.unlink(llvm_file)
            
            return BenchmarkResult('cpp', test_name, compilation_time, None, success, output_size)
            
        finally:
            os.unlink(temp_file)
    
    def benchmark_go_compiler(self, program: str, test_name: str) -> BenchmarkResult:
        """Benchmark the Go compiler."""
        if not self.compilers['go'].available:
            return BenchmarkResult('go', test_name, 0, None, False, None)
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.go', delete=False) as f:
            f.write(program)
            temp_file = f.name
        
        try:
            start_time = time.time()
            result = subprocess.run(
                ['go', 'build', '-o', temp_file.replace('.go', ''), temp_file],
                capture_output=True,
                text=True
            )
            compilation_time = (time.time() - start_time) * 1000
            
            success = result.returncode == 0
            output_size = None
            
            binary_file = temp_file.replace('.go', '')
            if os.path.exists(binary_file):
                output_size = os.path.getsize(binary_file)
                os.unlink(binary_file)
            
            return BenchmarkResult('go', test_name, compilation_time, None, success, output_size)
            
        finally:
            os.unlink(temp_file)
    
    def run_comprehensive_benchmarks(self) -> None:
        """Run all benchmark tests."""
        print("🔧 Running Evidence-Based Performance Validation...")
        print(f"Available compilers: {[name for name, info in self.compilers.items() if info.available]}")
        
        test_programs = self.create_test_programs()
        
        for test_name, programs in test_programs.items():
            print(f"\n📊 Benchmarking {test_name}...")
            
            # Run multiple iterations for statistical significance
            iterations = 5
            
            for compiler, program in programs.items():
                if not self.compilers[compiler].available:
                    continue
                
                iteration_results = []
                
                for i in range(iterations):
                    if compiler == 'ea':
                        result = self.benchmark_ea_compiler(program, test_name)
                    elif compiler == 'rust':
                        result = self.benchmark_rust_compiler(program, test_name)
                    elif compiler == 'cpp':
                        result = self.benchmark_cpp_compiler(program, test_name)
                    elif compiler == 'go':
                        result = self.benchmark_go_compiler(program, test_name)
                    
                    if result.success:
                        iteration_results.append(result.compilation_time_ms)
                
                if iteration_results:
                    # Calculate statistics
                    mean_time = statistics.mean(iteration_results)
                    median_time = statistics.median(iteration_results)
                    std_dev = statistics.stdev(iteration_results) if len(iteration_results) > 1 else 0
                    
                    # Store the median result as representative
                    final_result = BenchmarkResult(
                        compiler, test_name, median_time, None, True, result.output_size
                    )
                    self.results.append(final_result)
                    
                    print(f"  {compiler:>6}: {median_time:>8.2f}ms (±{std_dev:>6.2f}ms)")
                else:
                    print(f"  {compiler:>6}: FAILED")
    
    def generate_performance_report(self) -> str:
        """Generate a comprehensive performance report."""
        report = []
        report.append("# Evidence-Based Performance Validation Report")
        report.append(f"Generated on: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")
        
        # Compiler versions
        report.append("## Compiler Versions")
        for name, info in self.compilers.items():
            status = "✅ Available" if info.available else "❌ Not Available"
            report.append(f"- **{info.name}**: {info.version} ({status})")
        report.append("")
        
        # Group results by test
        test_groups = {}
        for result in self.results:
            if result.test_name not in test_groups:
                test_groups[result.test_name] = []
            test_groups[result.test_name].append(result)
        
        # Performance comparison tables
        report.append("## Compilation Speed Results")
        report.append("")
        
        for test_name, results in test_groups.items():
            report.append(f"### {test_name.title()} Test")
            report.append("")
            report.append("| Compiler | Compilation Time | Relative Performance |")
            report.append("|----------|------------------|----------------------|")
            
            # Sort by compilation time
            results.sort(key=lambda r: r.compilation_time_ms)
            fastest_time = results[0].compilation_time_ms
            
            for result in results:
                relative_perf = f"{result.compilation_time_ms / fastest_time:.2f}x"
                if result.compiler == 'ea':
                    compiler_name = f"**{result.compiler}** (Eä)"
                else:
                    compiler_name = result.compiler
                report.append(f"| {compiler_name} | {result.compilation_time_ms:.2f}ms | {relative_perf} |")
            
            report.append("")
        
        # Summary and analysis
        report.append("## Performance Analysis")
        report.append("")
        
        # Find Eä's performance relative to others
        ea_results = [r for r in self.results if r.compiler == 'ea']
        if ea_results:
            report.append("### Eä Compiler Performance Summary")
            report.append("")
            
            for ea_result in ea_results:
                test_results = [r for r in self.results if r.test_name == ea_result.test_name]
                test_results.sort(key=lambda r: r.compilation_time_ms)
                
                ea_rank = next(i for i, r in enumerate(test_results) if r.compiler == 'ea') + 1
                total_compilers = len(test_results)
                
                fastest = test_results[0]
                if fastest.compiler == 'ea':
                    status = "🥇 **FASTEST**"
                else:
                    ratio = ea_result.compilation_time_ms / fastest.compilation_time_ms
                    if ratio <= 1.5:
                        status = f"🥈 Very competitive ({ratio:.2f}x slower)"
                    elif ratio <= 2.0:
                        status = f"🥉 Competitive ({ratio:.2f}x slower)"
                    else:
                        status = f"📊 Needs improvement ({ratio:.2f}x slower)"
                
                report.append(f"- **{ea_result.test_name}**: Rank {ea_rank}/{total_compilers} - {status}")
            
            report.append("")
        
        # Technical details
        report.append("## Technical Notes")
        report.append("")
        report.append("- All tests run 5 iterations, median time reported")
        report.append("- Compilation includes full pipeline to LLVM IR where applicable")
        report.append("- Tests use equivalent algorithms across all languages")
        report.append("- Memory usage measurements require additional tooling (not included)")
        report.append("")
        
        # Recommendations
        report.append("## Recommendations")
        report.append("")
        ea_results = [r for r in self.results if r.compiler == 'ea']
        if ea_results:
            avg_ratio = statistics.mean([
                min(r.compilation_time_ms for r in self.results if r.test_name == ea_r.test_name and r.compiler != 'ea') / ea_r.compilation_time_ms
                for ea_r in ea_results
                if any(r.test_name == ea_r.test_name and r.compiler != 'ea' for r in self.results)
            ])
            
            if avg_ratio > 1.0:
                report.append(f"✅ Eä is **{avg_ratio:.2f}x faster** than competitors on average")
                report.append("- Continue optimizing to maintain performance leadership")
                report.append("- Focus on memory efficiency improvements")
            else:
                report.append(f"⚠️ Eä is **{1/avg_ratio:.2f}x slower** than competitors on average")
                report.append("- Priority: Optimize compilation pipeline performance")
                report.append("- Focus on lexer and parser speed improvements")
                report.append("- Investigate LLVM IR generation efficiency")
        
        return "\n".join(report)
    
    def save_results(self, filename: str) -> None:
        """Save benchmark results to JSON file."""
        data = {
            'timestamp': time.time(),
            'compilers': {name: {'name': info.name, 'version': info.version, 'available': info.available} 
                         for name, info in self.compilers.items()},
            'results': [
                {
                    'compiler': r.compiler,
                    'test_name': r.test_name,
                    'compilation_time_ms': r.compilation_time_ms,
                    'memory_usage_mb': r.memory_usage_mb,
                    'success': r.success,
                    'output_size_bytes': r.output_size_bytes
                }
                for r in self.results
            ]
        }
        
        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)

def main():
    """Main entry point for performance validation."""
    validator = PerformanceValidator()
    
    print("🚀 Eä Compiler Evidence-Based Performance Validation")
    print("=" * 55)
    
    # Run benchmarks
    validator.run_comprehensive_benchmarks()
    
    # Generate and save report
    report = validator.generate_performance_report()
    
    # Save results
    results_dir = Path(__file__).parent.parent / 'benchmark_results'
    results_dir.mkdir(exist_ok=True)
    
    timestamp = time.strftime('%Y%m%d_%H%M%S')
    validator.save_results(results_dir / f'performance_results_{timestamp}.json')
    
    # Save markdown report
    report_file = results_dir / f'performance_report_{timestamp}.md'
    with open(report_file, 'w') as f:
        f.write(report)
    
    print(f"\n📋 Performance report saved to: {report_file}")
    print(f"📊 Raw results saved to: results_dir / f'performance_results_{timestamp}.json'")
    
    # Print summary
    print("\n" + "=" * 55)
    print(report)

if __name__ == '__main__':
    main()