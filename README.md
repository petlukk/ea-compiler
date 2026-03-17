# ea-compiler

Python package wrapping the Eä compute kernel compiler. Provides `ea.load("kernel.ea")` which compiles, caches, and returns callable kernels via ctypes.

## Install

```bash
pip install ea-compiler
```

## Usage

```python
import ea

kernel = ea.load("scale.ea")
result = kernel.scale(src_array, 2.0)
```
