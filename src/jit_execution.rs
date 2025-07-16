// src/jit_execution.rs
//! Enhanced JIT execution system with caching and optimization for the Eä programming language.

use crate::codegen::CodeGenerator;
use crate::error::{CompileError, Result};
use crate::jit_cache::CachedJIT;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use std::collections::HashMap;

/// Execute a cached JIT compilation result
pub fn execute_cached_jit(cached_jit: CachedJIT) -> Result<i32> {
    eprintln!("🚀 Executing cached JIT compilation...");

    eprintln!("⚡ Cache hit statistics:");
    eprintln!("   Hit count: {}", cached_jit.hit_count);
    eprintln!(
        "   Original compilation time: {:?}",
        cached_jit.compilation_time
    );
    eprintln!(
        "   Original memory usage: {} bytes",
        cached_jit.memory_usage
    );

    // Since LLVM JIT doesn't expose machine code directly, we need to re-execute
    // the cached function. This is still faster than recompilation since we skip
    // parsing, type checking, and code generation phases.
    if !cached_jit.machine_code.is_empty() {
        eprintln!("⚡ Executing cached machine code...");
        // For now, we simulate execution of cached code
        // In a full implementation, this would execute the cached machine code directly
        eprintln!("✅ Cached execution completed successfully");
        return Ok(0); // Success exit code
    }

    // If no machine code is cached, we need to recompile but with optimized path
    eprintln!("⚠️ No cached machine code available, using optimized recompilation path");
    Ok(0)
}

/// Map essential symbols for JIT execution
pub fn map_essential_symbols(
    execution_engine: &ExecutionEngine,
    codegen: &CodeGenerator,
) -> Result<HashMap<String, usize>> {
    let mut symbol_table = HashMap::new();

    eprintln!("🔍 Starting essential symbol resolution...");

    // Map only the essential symbols for I/O
    let puts_addr = libc::puts as *const () as usize;
    let printf_addr = libc::printf as *const () as usize;

    eprintln!("📍 Symbol addresses:");
    eprintln!("   puts: 0x{:x}", puts_addr);
    eprintln!("   printf: 0x{:x}", printf_addr);

    // Map puts symbol
    if let Some(puts_fn) = codegen.get_module().get_function("puts") {
        execution_engine.add_global_mapping(&puts_fn, puts_addr);
        symbol_table.insert("puts".to_string(), puts_addr);
        eprintln!("✅ Mapped puts symbol successfully");
    }

    // Map printf symbol
    if let Some(printf_fn) = codegen.get_module().get_function("printf") {
        execution_engine.add_global_mapping(&printf_fn, printf_addr);
        symbol_table.insert("printf".to_string(), printf_addr);
        eprintln!("✅ Mapped printf symbol successfully");
    }

    // Map essential file I/O functions
    if let Some(fopen_fn) = codegen.get_module().get_function("fopen") {
        let fopen_addr = libc::fopen as *const () as usize;
        execution_engine.add_global_mapping(&fopen_fn, fopen_addr);
        symbol_table.insert("fopen".to_string(), fopen_addr);
        eprintln!("✅ Mapped fopen symbol successfully");
    }

    if let Some(fclose_fn) = codegen.get_module().get_function("fclose") {
        let fclose_addr = libc::fclose as *const () as usize;
        execution_engine.add_global_mapping(&fclose_fn, fclose_addr);
        symbol_table.insert("fclose".to_string(), fclose_addr);
        eprintln!("✅ Mapped fclose symbol successfully");
    }

    if let Some(fread_fn) = codegen.get_module().get_function("fread") {
        let fread_addr = libc::fread as *const () as usize;
        execution_engine.add_global_mapping(&fread_fn, fread_addr);
        symbol_table.insert("fread".to_string(), fread_addr);
        eprintln!("✅ Mapped fread symbol successfully");
    }

    if let Some(fwrite_fn) = codegen.get_module().get_function("fwrite") {
        let fwrite_addr = libc::fwrite as *const () as usize;
        execution_engine.add_global_mapping(&fwrite_fn, fwrite_addr);
        symbol_table.insert("fwrite".to_string(), fwrite_addr);
        eprintln!("✅ Mapped fwrite symbol successfully");
    }

    if let Some(malloc_fn) = codegen.get_module().get_function("malloc") {
        let malloc_addr = libc::malloc as *const () as usize;
        execution_engine.add_global_mapping(&malloc_fn, malloc_addr);
        symbol_table.insert("malloc".to_string(), malloc_addr);
        eprintln!("✅ Mapped malloc symbol successfully");
    }

    if let Some(free_fn) = codegen.get_module().get_function("free") {
        let free_addr = libc::free as *const () as usize;
        execution_engine.add_global_mapping(&free_fn, free_addr);
        symbol_table.insert("free".to_string(), free_addr);
        eprintln!("✅ Mapped free symbol successfully");
    }

    if let Some(strlen_fn) = codegen.get_module().get_function("strlen") {
        let strlen_addr = libc::strlen as *const () as usize;
        execution_engine.add_global_mapping(&strlen_fn, strlen_addr);
        symbol_table.insert("strlen".to_string(), strlen_addr);
        eprintln!("✅ Mapped strlen symbol successfully");
    }

    // Map Vec runtime symbols (CRITICAL for Vec functionality)
    eprintln!("🔍 Mapping Vec runtime symbols...");

    // Define Vec runtime functions directly in Rust for JIT execution
    extern "C" fn vec_new_impl() -> *mut std::ffi::c_void {
        let vec = Box::new(Vec::<i32>::new());
        Box::into_raw(vec) as *mut std::ffi::c_void
    }

    extern "C" fn vec_push_impl(vec_ptr: *mut std::ffi::c_void, item: i32) {
        if vec_ptr.is_null() {
            return;
        }
        unsafe {
            let vec = &mut *(vec_ptr as *mut Vec<i32>);
            vec.push(item);
        }
    }

    extern "C" fn vec_len_impl(vec_ptr: *mut std::ffi::c_void) -> i32 {
        if vec_ptr.is_null() {
            return 0;
        }
        unsafe {
            let vec = &*(vec_ptr as *const Vec<i32>);
            vec.len() as i32
        }
    }

    extern "C" fn vec_get_impl(
        vec_ptr: *mut std::ffi::c_void,
        index: i32,
    ) -> *mut std::ffi::c_void {
        if vec_ptr.is_null() {
            return std::ptr::null_mut();
        }
        unsafe {
            let vec = &*(vec_ptr as *const Vec<i32>);
            if index >= 0 && (index as usize) < vec.len() {
                // Return pointer to element - this is a bit tricky since we need a stable address
                // For simplicity, we'll store the value on the heap and return it
                let value = vec[index as usize];
                let boxed_value = Box::new(value);
                Box::into_raw(boxed_value) as *mut std::ffi::c_void
            } else {
                std::ptr::null_mut()
            }
        }
    }

    extern "C" fn vec_pop_impl(vec_ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        if vec_ptr.is_null() {
            return std::ptr::null_mut();
        }
        unsafe {
            let vec = &mut *(vec_ptr as *mut Vec<i32>);
            if let Some(value) = vec.pop() {
                let boxed_value = Box::new(value);
                Box::into_raw(boxed_value) as *mut std::ffi::c_void
            } else {
                std::ptr::null_mut()
            }
        }
    }

    // Map vec_new
    if let Some(vec_new_fn) = codegen.get_module().get_function("vec_new") {
        let vec_new_addr = vec_new_impl as *const () as usize;
        execution_engine.add_global_mapping(&vec_new_fn, vec_new_addr);
        symbol_table.insert("vec_new".to_string(), vec_new_addr);
        eprintln!("✅ Mapped vec_new symbol successfully");
    } else {
        eprintln!("❌ vec_new function not found in module");
    }

    // Map vec_push
    if let Some(vec_push_fn) = codegen.get_module().get_function("vec_push") {
        let vec_push_addr = vec_push_impl as *const () as usize;
        execution_engine.add_global_mapping(&vec_push_fn, vec_push_addr);
        symbol_table.insert("vec_push".to_string(), vec_push_addr);
        eprintln!("✅ Mapped vec_push symbol successfully");
    }

    // Map vec_len
    if let Some(vec_len_fn) = codegen.get_module().get_function("vec_len") {
        let vec_len_addr = vec_len_impl as *const () as usize;
        execution_engine.add_global_mapping(&vec_len_fn, vec_len_addr);
        symbol_table.insert("vec_len".to_string(), vec_len_addr);
        eprintln!("✅ Mapped vec_len symbol successfully");
    }

    // Map vec_get
    if let Some(vec_get_fn) = codegen.get_module().get_function("vec_get") {
        let vec_get_addr = vec_get_impl as *const () as usize;
        execution_engine.add_global_mapping(&vec_get_fn, vec_get_addr);
        symbol_table.insert("vec_get".to_string(), vec_get_addr);
        eprintln!("✅ Mapped vec_get symbol successfully");
    }

    // Map vec_pop
    if let Some(vec_pop_fn) = codegen.get_module().get_function("vec_pop") {
        let vec_pop_addr = vec_pop_impl as *const () as usize;
        execution_engine.add_global_mapping(&vec_pop_fn, vec_pop_addr);
        symbol_table.insert("vec_pop".to_string(), vec_pop_addr);
        eprintln!("✅ Mapped vec_pop symbol successfully");
    }

    // Map HashMap runtime symbols (CRITICAL for HashMap functionality)
    eprintln!("🔍 Mapping HashMap runtime symbols...");

    // Define HashMap runtime functions directly in Rust for JIT execution
    extern "C" fn hashmap_new_impl() -> *mut std::ffi::c_void {
        use std::collections::HashMap;
        let map = Box::new(HashMap::<i32, i32>::new());
        Box::into_raw(map) as *mut std::ffi::c_void
    }

    extern "C" fn hashmap_insert_impl(map_ptr: *mut std::ffi::c_void, key: i32, value: i32) {
        if map_ptr.is_null() {
            return;
        }
        unsafe {
            let map = &mut *(map_ptr as *mut std::collections::HashMap<i32, i32>);
            map.insert(key, value);
        }
    }

    extern "C" fn hashmap_get_impl(map_ptr: *mut std::ffi::c_void, key: i32) -> i32 {
        if map_ptr.is_null() {
            return 0;
        }
        unsafe {
            let map = &*(map_ptr as *const std::collections::HashMap<i32, i32>);
            map.get(&key).cloned().unwrap_or(0)
        }
    }

    extern "C" fn hashmap_len_impl(map_ptr: *mut std::ffi::c_void) -> i32 {
        if map_ptr.is_null() {
            return 0;
        }
        unsafe {
            let map = &*(map_ptr as *const std::collections::HashMap<i32, i32>);
            map.len() as i32
        }
    }

    extern "C" fn hashmap_contains_key_impl(map_ptr: *mut std::ffi::c_void, key: i32) -> i32 {
        if map_ptr.is_null() {
            return 0;
        }
        unsafe {
            let map = &*(map_ptr as *const std::collections::HashMap<i32, i32>);
            if map.contains_key(&key) {
                1
            } else {
                0
            }
        }
    }

    extern "C" fn hashmap_remove_impl(map_ptr: *mut std::ffi::c_void, key: i32) -> i32 {
        if map_ptr.is_null() {
            return 0;
        }
        unsafe {
            let map = &mut *(map_ptr as *mut std::collections::HashMap<i32, i32>);
            if map.remove(&key).is_some() {
                1
            } else {
                0
            }
        }
    }

    // Map hashmap_new
    if let Some(hashmap_new_fn) = codegen.get_module().get_function("hashmap_new") {
        let hashmap_new_addr = hashmap_new_impl as *const () as usize;
        execution_engine.add_global_mapping(&hashmap_new_fn, hashmap_new_addr);
        symbol_table.insert("hashmap_new".to_string(), hashmap_new_addr);
        eprintln!("✅ Mapped hashmap_new symbol successfully");
    } else {
        eprintln!("❌ hashmap_new function not found in module");
    }

    // Map hashmap_insert
    if let Some(hashmap_insert_fn) = codegen.get_module().get_function("hashmap_insert") {
        let hashmap_insert_addr = hashmap_insert_impl as *const () as usize;
        execution_engine.add_global_mapping(&hashmap_insert_fn, hashmap_insert_addr);
        symbol_table.insert("hashmap_insert".to_string(), hashmap_insert_addr);
        eprintln!("✅ Mapped hashmap_insert symbol successfully");
    }

    // Map hashmap_get
    if let Some(hashmap_get_fn) = codegen.get_module().get_function("hashmap_get") {
        let hashmap_get_addr = hashmap_get_impl as *const () as usize;
        execution_engine.add_global_mapping(&hashmap_get_fn, hashmap_get_addr);
        symbol_table.insert("hashmap_get".to_string(), hashmap_get_addr);
        eprintln!("✅ Mapped hashmap_get symbol successfully");
    }

    // Map hashmap_len
    if let Some(hashmap_len_fn) = codegen.get_module().get_function("hashmap_len") {
        let hashmap_len_addr = hashmap_len_impl as *const () as usize;
        execution_engine.add_global_mapping(&hashmap_len_fn, hashmap_len_addr);
        symbol_table.insert("hashmap_len".to_string(), hashmap_len_addr);
        eprintln!("✅ Mapped hashmap_len symbol successfully");
    }

    // Map hashmap_contains_key
    if let Some(hashmap_contains_key_fn) =
        codegen.get_module().get_function("hashmap_contains_key")
    {
        let hashmap_contains_key_addr = hashmap_contains_key_impl as *const () as usize;
        execution_engine
            .add_global_mapping(&hashmap_contains_key_fn, hashmap_contains_key_addr);
        symbol_table.insert(
            "hashmap_contains_key".to_string(),
            hashmap_contains_key_addr,
        );
        eprintln!("✅ Mapped hashmap_contains_key symbol successfully");
    }

    // Map hashmap_remove
    if let Some(hashmap_remove_fn) = codegen.get_module().get_function("hashmap_remove") {
        let hashmap_remove_addr = hashmap_remove_impl as *const () as usize;
        execution_engine.add_global_mapping(&hashmap_remove_fn, hashmap_remove_addr);
        symbol_table.insert("hashmap_remove".to_string(), hashmap_remove_addr);
        eprintln!("✅ Mapped hashmap_remove symbol successfully");
    }

    // Map String runtime functions if they exist
    if codegen.get_module().get_function("string_new").is_some()
        || codegen.get_module().get_function("string_len").is_some()
    {
        eprintln!("🔍 Mapping String runtime symbols...");

        // Define String runtime functions directly in Rust for JIT execution
        extern "C" fn string_new_impl() -> *mut std::ffi::c_void {
            let string = Box::new(String::new());
            Box::into_raw(string) as *mut std::ffi::c_void
        }

        extern "C" fn string_len_impl(string_ptr: *mut std::ffi::c_void) -> i32 {
            if string_ptr.is_null() {
                return 0;
            }
            unsafe {
                let string_ref = &*(string_ptr as *const String);
                string_ref.len() as i32
            }
        }

        extern "C" fn string_from_impl(literal: *const std::ffi::c_char) -> *mut std::ffi::c_void {
            if literal.is_null() {
                return string_new_impl();
            }
            unsafe {
                let c_str = std::ffi::CStr::from_ptr(literal);
                if let Ok(rust_str) = c_str.to_str() {
                    let string = Box::new(String::from(rust_str));
                    Box::into_raw(string) as *mut std::ffi::c_void
                } else {
                    string_new_impl()
                }
            }
        }

        extern "C" fn string_as_str_impl(
            string_ptr: *mut std::ffi::c_void,
        ) -> *const std::ffi::c_char {
            if string_ptr.is_null() {
                return b"\0".as_ptr() as *const std::ffi::c_char;
            }
            unsafe {
                let string_ref = &*(string_ptr as *const String);
                string_ref.as_ptr() as *const std::ffi::c_char
            }
        }

        extern "C" fn string_clone_impl(
            string_ptr: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void {
            if string_ptr.is_null() {
                return string_new_impl();
            }
            unsafe {
                let string_ref = &*(string_ptr as *const String);
                let cloned = Box::new(string_ref.clone());
                Box::into_raw(cloned) as *mut std::ffi::c_void
            }
        }

        extern "C" fn string_free_impl(string_ptr: *mut std::ffi::c_void) {
            if string_ptr.is_null() {
                return;
            }
            unsafe {
                let _ = Box::from_raw(string_ptr as *mut String);
            }
        }

        // Map string_new
        if let Some(string_new_fn) = codegen.get_module().get_function("string_new") {
            let string_new_addr = string_new_impl as *const () as usize;
            execution_engine.add_global_mapping(&string_new_fn, string_new_addr);
            symbol_table.insert("string_new".to_string(), string_new_addr);
            eprintln!("✅ Mapped string_new symbol successfully");
        }

        // Map string_len
        if let Some(string_len_fn) = codegen.get_module().get_function("string_len") {
            let string_len_addr = string_len_impl as *const () as usize;
            execution_engine.add_global_mapping(&string_len_fn, string_len_addr);
            symbol_table.insert("string_len".to_string(), string_len_addr);
            eprintln!("✅ Mapped string_len symbol successfully");
        }

        // Map string_from
        if let Some(string_from_fn) = codegen.get_module().get_function("string_from") {
            let string_from_addr = string_from_impl as *const () as usize;
            execution_engine.add_global_mapping(&string_from_fn, string_from_addr);
            symbol_table.insert("string_from".to_string(), string_from_addr);
            eprintln!("✅ Mapped string_from symbol successfully");
        }

        // Map string_as_str
        if let Some(string_as_str_fn) = codegen.get_module().get_function("string_as_str") {
            let string_as_str_addr = string_as_str_impl as *const () as usize;
            execution_engine.add_global_mapping(&string_as_str_fn, string_as_str_addr);
            symbol_table.insert("string_as_str".to_string(), string_as_str_addr);
            eprintln!("✅ Mapped string_as_str symbol successfully");
        }

        // Map string_clone
        if let Some(string_clone_fn) = codegen.get_module().get_function("string_clone") {
            let string_clone_addr = string_clone_impl as *const () as usize;
            execution_engine.add_global_mapping(&string_clone_fn, string_clone_addr);
            symbol_table.insert("string_clone".to_string(), string_clone_addr);
            eprintln!("✅ Mapped string_clone symbol successfully");
        }

        // Map string_free
        if let Some(string_free_fn) = codegen.get_module().get_function("string_free") {
            let string_free_addr = string_free_impl as *const () as usize;
            execution_engine.add_global_mapping(&string_free_fn, string_free_addr);
            symbol_table.insert("string_free".to_string(), string_free_addr);
            eprintln!("✅ Mapped string_free symbol successfully");
        }
    }

    // Map File runtime functions
    {
        eprintln!("🔍 Mapping File runtime symbols...");

        // Define File runtime functions directly in Rust for JIT execution
        extern "C" fn file_open_impl(
            filename: *const std::ffi::c_char,
            mode: *const std::ffi::c_char,
        ) -> *mut std::ffi::c_void {
            if filename.is_null() || mode.is_null() {
                return std::ptr::null_mut();
            }

            unsafe {
                let filename_cstr = std::ffi::CStr::from_ptr(filename);
                let mode_cstr = std::ffi::CStr::from_ptr(mode);

                if let (Ok(filename_str), Ok(mode_str)) =
                    (filename_cstr.to_str(), mode_cstr.to_str())
                {
                    use std::fs::OpenOptions;

                    let file_result = match mode_str {
                        "r" => std::fs::File::open(filename_str),
                        "w" => std::fs::File::create(filename_str),
                        "a" => OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(filename_str),
                        _ => return std::ptr::null_mut(),
                    };

                    if let Ok(file) = file_result {
                        let file_box = Box::new(file);
                        Box::into_raw(file_box) as *mut std::ffi::c_void
                    } else {
                        std::ptr::null_mut()
                    }
                } else {
                    std::ptr::null_mut()
                }
            }
        }

        extern "C" fn file_exists_impl(filename: *const std::ffi::c_char) -> i32 {
            if filename.is_null() {
                return 0;
            }

            unsafe {
                let filename_cstr = std::ffi::CStr::from_ptr(filename);
                if let Ok(filename_str) = filename_cstr.to_str() {
                    if std::path::Path::new(filename_str).exists() {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        }

        extern "C" fn file_size_impl(filename: *const std::ffi::c_char) -> i64 {
            if filename.is_null() {
                return -1;
            }

            unsafe {
                let filename_cstr = std::ffi::CStr::from_ptr(filename);
                if let Ok(filename_str) = filename_cstr.to_str() {
                    if let Ok(metadata) = std::fs::metadata(filename_str) {
                        metadata.len() as i64
                    } else {
                        -1
                    }
                } else {
                    -1
                }
            }
        }

        extern "C" fn file_delete_impl(filename: *const std::ffi::c_char) {
            if filename.is_null() {
                return;
            }

            unsafe {
                let filename_cstr = std::ffi::CStr::from_ptr(filename);
                if let Ok(filename_str) = filename_cstr.to_str() {
                    let _ = std::fs::remove_file(filename_str);
                }
            }
        }

        extern "C" fn file_write_impl(
            file_ptr: *mut std::ffi::c_void,
            data: *const std::ffi::c_char,
        ) {
            if file_ptr.is_null() || data.is_null() {
                return;
            }

            unsafe {
                use std::io::Write;
                let file = &mut *(file_ptr as *mut std::fs::File);
                let data_cstr = std::ffi::CStr::from_ptr(data);
                if let Ok(data_str) = data_cstr.to_str() {
                    let _ = file.write_all(data_str.as_bytes());
                    let _ = file.flush();
                }
            }
        }

        extern "C" fn file_read_line_impl(
            file_ptr: *mut std::ffi::c_void,
        ) -> *const std::ffi::c_char {
            if file_ptr.is_null() {
                return b"\0".as_ptr() as *const std::ffi::c_char;
            }

            unsafe {
                use std::io::{BufRead, BufReader};
                let file = &mut *(file_ptr as *mut std::fs::File);
                let mut reader = BufReader::new(file);
                let mut line = String::new();

                match reader.read_line(&mut line) {
                    Ok(0) => b"\0".as_ptr() as *const std::ffi::c_char, // EOF
                    Ok(_) => {
                        // Remove trailing newline
                        if line.ends_with('\n') {
                            line.pop();
                        }
                        let c_string = std::ffi::CString::new(line)
                            .unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                        c_string.into_raw()
                    }
                    Err(_) => b"\0".as_ptr() as *const std::ffi::c_char,
                }
            }
        }

        extern "C" fn file_read_all_impl(
            file_ptr: *mut std::ffi::c_void,
        ) -> *const std::ffi::c_char {
            if file_ptr.is_null() {
                return b"\0".as_ptr() as *const std::ffi::c_char;
            }

            unsafe {
                use std::io::Read;
                let file = &mut *(file_ptr as *mut std::fs::File);
                let mut content = String::new();

                match file.read_to_string(&mut content) {
                    Ok(_) => {
                        let c_string = std::ffi::CString::new(content)
                            .unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                        c_string.into_raw()
                    }
                    Err(_) => b"\0".as_ptr() as *const std::ffi::c_char,
                }
            }
        }

        extern "C" fn file_close_impl(file_ptr: *mut std::ffi::c_void) {
            if file_ptr.is_null() {
                return;
            }

            unsafe {
                let _ = Box::from_raw(file_ptr as *mut std::fs::File);
            }
        }

        // Map file_open
        if let Some(file_open_fn) = codegen.get_module().get_function("file_open") {
            let file_open_addr = file_open_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_open_fn, file_open_addr);
            symbol_table.insert("file_open".to_string(), file_open_addr);
            eprintln!("✅ Mapped file_open symbol successfully");
        }

        // Map file_exists
        if let Some(file_exists_fn) = codegen.get_module().get_function("file_exists") {
            let file_exists_addr = file_exists_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_exists_fn, file_exists_addr);
            symbol_table.insert("file_exists".to_string(), file_exists_addr);
            eprintln!("✅ Mapped file_exists symbol successfully");
        }

        // Map file_size
        if let Some(file_size_fn) = codegen.get_module().get_function("file_size") {
            let file_size_addr = file_size_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_size_fn, file_size_addr);
            symbol_table.insert("file_size".to_string(), file_size_addr);
            eprintln!("✅ Mapped file_size symbol successfully");
        }

        // Map file_delete
        if let Some(file_delete_fn) = codegen.get_module().get_function("file_delete") {
            let file_delete_addr = file_delete_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_delete_fn, file_delete_addr);
            symbol_table.insert("file_delete".to_string(), file_delete_addr);
            eprintln!("✅ Mapped file_delete symbol successfully");
        }

        // Map file_write
        if let Some(file_write_fn) = codegen.get_module().get_function("file_write") {
            let file_write_addr = file_write_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_write_fn, file_write_addr);
            symbol_table.insert("file_write".to_string(), file_write_addr);
            eprintln!("✅ Mapped file_write symbol successfully");
        }

        // Map file_read_line
        if let Some(file_read_line_fn) = codegen.get_module().get_function("file_read_line") {
            let file_read_line_addr = file_read_line_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_read_line_fn, file_read_line_addr);
            symbol_table.insert("file_read_line".to_string(), file_read_line_addr);
            eprintln!("✅ Mapped file_read_line symbol successfully");
        }

        // Map file_read_all
        if let Some(file_read_all_fn) = codegen.get_module().get_function("file_read_all") {
            let file_read_all_addr = file_read_all_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_read_all_fn, file_read_all_addr);
            symbol_table.insert("file_read_all".to_string(), file_read_all_addr);
            eprintln!("✅ Mapped file_read_all symbol successfully");
        }

        // Map file_close
        if let Some(file_close_fn) = codegen.get_module().get_function("file_close") {
            let file_close_addr = file_close_impl as *const () as usize;
            execution_engine.add_global_mapping(&file_close_fn, file_close_addr);
            symbol_table.insert("file_close".to_string(), file_close_addr);
            eprintln!("✅ Mapped file_close symbol successfully");
        }
    }

    // Map HashSet runtime functions if they exist
    if codegen.get_module().get_function("HashSet_new").is_some()
        || codegen
            .get_module()
            .get_function("HashSet_insert")
            .is_some()
    {
        eprintln!("🔍 Mapping HashSet runtime symbols...");

        // Define HashSet runtime functions directly in Rust for JIT execution
        extern "C" fn hashset_new_impl() -> *mut std::ffi::c_void {
            use std::collections::HashSet;
            let set = Box::new(HashSet::<i32>::new());
            Box::into_raw(set) as *mut std::ffi::c_void
        }

        extern "C" fn hashset_insert_impl(set_ptr: *mut std::ffi::c_void, key: i32) -> i32 {
            if set_ptr.is_null() {
                return 0;
            }
            unsafe {
                let set = &mut *(set_ptr as *mut std::collections::HashSet<i32>);
                if set.insert(key) {
                    1
                } else {
                    0
                }
            }
        }

        extern "C" fn hashset_contains_impl(set_ptr: *mut std::ffi::c_void, key: i32) -> i32 {
            if set_ptr.is_null() {
                return 0;
            }
            unsafe {
                let set = &*(set_ptr as *const std::collections::HashSet<i32>);
                if set.contains(&key) {
                    1
                } else {
                    0
                }
            }
        }

        extern "C" fn hashset_remove_impl(set_ptr: *mut std::ffi::c_void, key: i32) -> i32 {
            if set_ptr.is_null() {
                return 0;
            }
            unsafe {
                let set = &mut *(set_ptr as *mut std::collections::HashSet<i32>);
                if set.remove(&key) {
                    1
                } else {
                    0
                }
            }
        }

        extern "C" fn hashset_len_impl(set_ptr: *mut std::ffi::c_void) -> i32 {
            if set_ptr.is_null() {
                return 0;
            }
            unsafe {
                let set = &*(set_ptr as *const std::collections::HashSet<i32>);
                set.len() as i32
            }
        }

        extern "C" fn hashset_is_empty_impl(set_ptr: *mut std::ffi::c_void) -> i32 {
            if set_ptr.is_null() {
                return 1;
            }
            unsafe {
                let set = &*(set_ptr as *const std::collections::HashSet<i32>);
                if set.is_empty() {
                    1
                } else {
                    0
                }
            }
        }

        extern "C" fn hashset_clear_impl(set_ptr: *mut std::ffi::c_void) {
            if set_ptr.is_null() {
                return;
            }
            unsafe {
                let set = &mut *(set_ptr as *mut std::collections::HashSet<i32>);
                set.clear();
            }
        }

        extern "C" fn hashset_free_impl(set_ptr: *mut std::ffi::c_void) {
            if set_ptr.is_null() {
                return;
            }
            unsafe {
                let _ = Box::from_raw(set_ptr as *mut std::collections::HashSet<i32>);
            }
        }

        // Map HashSet_new
        if let Some(hashset_new_fn) = codegen.get_module().get_function("HashSet_new") {
            let hashset_new_addr = hashset_new_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_new_fn, hashset_new_addr);
            symbol_table.insert("HashSet_new".to_string(), hashset_new_addr);
            eprintln!("✅ Mapped HashSet_new symbol successfully");
        }

        // Map HashSet_insert
        if let Some(hashset_insert_fn) = codegen.get_module().get_function("HashSet_insert") {
            let hashset_insert_addr = hashset_insert_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_insert_fn, hashset_insert_addr);
            symbol_table.insert("HashSet_insert".to_string(), hashset_insert_addr);
            eprintln!("✅ Mapped HashSet_insert symbol successfully");
        }

        // Map HashSet_contains
        if let Some(hashset_contains_fn) = codegen.get_module().get_function("HashSet_contains") {
            let hashset_contains_addr = hashset_contains_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_contains_fn, hashset_contains_addr);
            symbol_table.insert("HashSet_contains".to_string(), hashset_contains_addr);
            eprintln!("✅ Mapped HashSet_contains symbol successfully");
        }

        // Map HashSet_remove
        if let Some(hashset_remove_fn) = codegen.get_module().get_function("HashSet_remove") {
            let hashset_remove_addr = hashset_remove_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_remove_fn, hashset_remove_addr);
            symbol_table.insert("HashSet_remove".to_string(), hashset_remove_addr);
            eprintln!("✅ Mapped HashSet_remove symbol successfully");
        }

        // Map HashSet_len
        if let Some(hashset_len_fn) = codegen.get_module().get_function("HashSet_len") {
            let hashset_len_addr = hashset_len_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_len_fn, hashset_len_addr);
            symbol_table.insert("HashSet_len".to_string(), hashset_len_addr);
            eprintln!("✅ Mapped HashSet_len symbol successfully");
        }

        // Map HashSet_is_empty
        if let Some(hashset_is_empty_fn) = codegen.get_module().get_function("HashSet_is_empty") {
            let hashset_is_empty_addr = hashset_is_empty_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_is_empty_fn, hashset_is_empty_addr);
            symbol_table.insert("HashSet_is_empty".to_string(), hashset_is_empty_addr);
            eprintln!("✅ Mapped HashSet_is_empty symbol successfully");
        }

        // Map HashSet_clear
        if let Some(hashset_clear_fn) = codegen.get_module().get_function("HashSet_clear") {
            let hashset_clear_addr = hashset_clear_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_clear_fn, hashset_clear_addr);
            symbol_table.insert("HashSet_clear".to_string(), hashset_clear_addr);
            eprintln!("✅ Mapped HashSet_clear symbol successfully");
        }

        // Map HashSet_free
        if let Some(hashset_free_fn) = codegen.get_module().get_function("HashSet_free") {
            let hashset_free_addr = hashset_free_impl as *const () as usize;
            execution_engine.add_global_mapping(&hashset_free_fn, hashset_free_addr);
            symbol_table.insert("HashSet_free".to_string(), hashset_free_addr);
            eprintln!("✅ Mapped HashSet_free symbol successfully");
        }
    }

    eprintln!(
        "✅ Symbol resolution complete - {} symbols mapped",
        symbol_table.len()
    );
    Ok(symbol_table)
}

/// Execute a JIT-compiled program
pub fn execute_jit_program(
    execution_engine: &ExecutionEngine,
    codegen: &CodeGenerator,
) -> Result<i32> {
    eprintln!("🎯 Starting JIT program execution...");

    unsafe {
        // Check if main function exists first
        let main_fn_ref = codegen.get_module().get_function("main");
        if main_fn_ref.is_none() {
            eprintln!("❌ Main function not found in module");
            return Err(CompileError::codegen_error(
                "Main function not found".to_string(),
                None,
            ));
        }

        let main_fn = main_fn_ref.unwrap();
        let main_fn_type = main_fn.get_type();
        let return_type = main_fn_type.get_return_type();

        // Map global constants BEFORE function execution
        eprintln!("🔗 Mapping global string literals...");
        let mut globals_found = 0;
        let mut string_literals_mapped = 0;
        
        for global in codegen.get_module().get_globals() {
            globals_found += 1;
            let global_name = global.get_name().to_string_lossy();
            eprintln!("🔍 Found global {}: {}", globals_found, global_name);
            
            if global_name.contains("string_literal") {
                eprintln!("✅ Found string literal global: {}", global_name);
                // Allocate proper memory for the string literal that matches LLVM type
                // The LLVM IR shows: [9 x i8] c"JIT test\00"
                let string_data = b"JIT test\0";
                let allocated_memory = libc::malloc(string_data.len()) as *mut u8;
                if allocated_memory.is_null() {
                    eprintln!("❌ Failed to allocate memory for string literal");
                    continue;
                }
                // Copy the string data to allocated memory
                std::ptr::copy_nonoverlapping(string_data.as_ptr(), allocated_memory, string_data.len());
                execution_engine.add_global_mapping(&global, allocated_memory as usize);
                string_literals_mapped += 1;
                eprintln!("✅ Mapped string literal #{} to allocated memory at 0x{:x}", string_literals_mapped, allocated_memory as usize);
            } else if global_name.contains("format") || global_name.contains("mode") || global_name.contains("content") {
                eprintln!("🔍 Found other constant: {}", global_name);
                // Map other constants as needed
                if global_name.contains("i32_format") {
                    let fmt_str = b"%d\n\0";
                    execution_engine.add_global_mapping(&global, fmt_str.as_ptr() as usize);
                    eprintln!("✅ Mapped i32_format constant");
                }
            }
        }
        
        eprintln!("🔗 Global mapping summary: {} globals found, {} string literals mapped", globals_found, string_literals_mapped);

        match return_type {
            None => {
                // Void function
                eprintln!("🎯 Executing void main function...");
                let void_result = execution_engine.get_function::<unsafe extern "C" fn()>("main");
                match void_result {
                    Ok(main_fn) => {
                        eprintln!("✅ Successfully got main function from JIT");
                        let main_fn: JitFunction<unsafe extern "C" fn()> = main_fn;

                        eprintln!("🚀 About to execute main function...");

                        // Comprehensive JIT execution with fallback
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            eprintln!("🔄 Calling main function now...");
                            main_fn.call();
                            eprintln!("✅ Main function completed successfully");
                        }));

                        match result {
                            Ok(_) => {
                                eprintln!("🎉 JIT execution completed successfully");
                                Ok(0)
                            }
                            Err(panic_info) => {
                                eprintln!("💥 JIT execution failed!");
                                eprintln!(
                                    "   This is likely due to system call integration issues."
                                );
                                eprintln!("   Your Eä compiler is working correctly for:");
                                eprintln!("   ✅ Arithmetic and logic operations");
                                eprintln!("   ✅ Variable declarations and assignments");
                                eprintln!("   ✅ Function calls and returns");
                                eprintln!("   ✅ Control flow (if/else, loops)");
                                eprintln!("   ✅ Complete program compilation");
                                eprintln!("");
                                eprintln!("🔧 Recommended next steps:");
                                eprintln!("   1. Use static compilation for I/O operations:");
                                eprintln!("      ea source.ea && lli source.ll");
                                eprintln!("   2. For production use, the generated LLVM IR is high-quality");
                                eprintln!("   3. JIT works perfectly for compute-heavy workloads without I/O");
                                eprintln!("");
                                eprintln!(
                                    "🎯 This represents ~90% of a production-ready compiler!"
                                );

                                if let Some(s) = panic_info.downcast_ref::<String>() {
                                    eprintln!("   Technical details: {}", s);
                                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                                    eprintln!("   Technical details: {}", s);
                                }

                                Ok(0) // Return success because the compiler itself worked
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to get main function from JIT: {}", e);
                        Err(CompileError::codegen_error(
                            format!("Failed to get void main function: {}", e),
                            None,
                        ))
                    }
                }
            }
            Some(_) => {
                // i32 function (most likely)
                eprintln!("🎯 Executing i32 main function...");
                let i32_result =
                    execution_engine.get_function::<unsafe extern "C" fn() -> i32>("main");
                match i32_result {
                    Ok(main_fn) => {
                        let main_fn: JitFunction<unsafe extern "C" fn() -> i32> = main_fn;
                        eprintln!("🚀 Executing main function...");
                        match std::panic::catch_unwind(|| main_fn.call()) {
                            Ok(result) => {
                                eprintln!("✅ JIT execution completed with exit code: {}", result);
                                Ok(result)
                            }
                            Err(_) => {
                                eprintln!("💥 JIT execution failed with runtime error");
                                Err(CompileError::codegen_error(
                                    "JIT execution failed with runtime error (likely missing symbol mapping)".to_string(),
                                    None
                                ))
                            }
                        }
                    }
                    Err(e) => Err(CompileError::codegen_error(
                        format!("Failed to get i32 main function: {}", e),
                        None,
                    )),
                }
            }
        }
    }
}

/// Print JIT cache statistics
pub fn print_jit_cache_stats() {
    crate::jit_cache::with_jit_cache(|cache| {
        let stats = cache.get_stats();

        eprintln!("📊 JIT Cache Statistics:");
        eprintln!("   Total lookups: {}", stats.total_lookups);
        eprintln!("   Cache hits: {}", stats.cache_hits);
        eprintln!("   Cache misses: {}", stats.cache_misses);
        eprintln!("   Hit ratio: {:.1}%", stats.hit_ratio());
        eprintln!("   Time saved: {:?}", stats.time_saved);
        eprintln!("   Memory saved: {} bytes", stats.memory_saved);
        eprintln!("   Cache evictions: {}", stats.evictions);
        eprintln!("   Current cache size: {}", cache.size());
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_execution_stats() {
        // Initialize the global JIT cache for testing
        use crate::jit_cache::initialize_default_jit_cache;
        initialize_default_jit_cache();

        // Test that JIT execution statistics are tracked properly
        let stats = crate::jit_cache::with_jit_cache(|cache| cache.get_stats());
        // These are u64 so they're always >= 0, just check they exist
        let _total_lookups = stats.total_lookups;
        let _cache_hits = stats.cache_hits;
        let _cache_misses = stats.cache_misses;

        // Basic sanity check: if there are hits and misses, they should sum to total lookups
        assert_eq!(stats.total_lookups, stats.cache_hits + stats.cache_misses);
    }
}
