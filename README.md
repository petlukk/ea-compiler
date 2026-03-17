# ea-compiler

Python package wrapping the [Eä](https://github.com/petlukk/eacompute) compute kernel compiler. Write `.ea` kernel files, compile to native SIMD code, call from Python with NumPy arrays. No C toolchain, no Cython, no JIT warmup.

## Install

```bash
pip install ea-compiler
```

Works on Linux x86_64, Linux aarch64, and Windows x86_64.

## Quick start

Write a kernel (`scale.ea`):

```
export func scale(src: *f32, dst: *mut f32, factor: f32, n: i32) {
    let s: f32x8 = splat(factor)
    let mut i: i32 = 0
    while i < n {
        let v: f32x8 = load(src, i)
        store(dst, i, v .* s)
        i = i + 8
    }
}
```

Call it from Python:

```python
import ea
import numpy as np

kernel = ea.load("scale.ea")

src = np.random.randn(1_000_000).astype(np.float32)
dst = np.empty_like(src)
kernel.scale(src, dst, factor=2.0)
```

`ea.load()` compiles the kernel to a native shared library, caches it in `__eacache__/`, and returns a callable object with ctypes bindings generated from the function signature. Subsequent calls use the cache.

## API

```python
import ea

# Compile, cache, and load — returns callable kernel module
kernel = ea.load("kernel.ea", target="native", opt_level=3)

# Compile without caching — returns Path to .so/.dll
ea.compile("kernel.ea")

# Clear cached compilations
ea.clear_cache()

# Version info
ea.__version__          # package version
ea.compiler_version()   # bundled ea binary version
```

## How it works

1. Bundles a pre-built `ea` compiler binary (no Rust or LLVM needed)
2. `ea.load()` invokes the compiler via subprocess to produce a `.so`/`.dll`
3. Parses the generated `.ea.json` metadata to build ctypes bindings at runtime
4. Length parameters (`n`, `len`, `count`, etc.) are auto-filled from array sizes
5. Output parameters with `[cap: ...]` annotations are auto-allocated and returned
6. Results are cached per CPU target and compiler version

## Documentation

Full documentation: [petlukk.github.io/eacompute](https://petlukk.github.io/eacompute/)

## License

MIT
