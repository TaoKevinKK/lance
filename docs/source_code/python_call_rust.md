## FFI
FFI（Foreign Function Interface）是一种机制，它允许一种编程语言调用另一种语言编写的函数，而不需要重新实现或改写已有逻辑。在 Python 调用 Rust（或其他语言如 C/C++）时，FFI 是底层基础.
1. 语言 ABI 兼容
不同语言在调用函数时，其参数传递方式、返回值、内存布局、调用约定等需遵循统一标准（即 ABI，Application Binary Interface）。
C ABI 是跨语言 FFI 的“通用语言”，Rust、C、C++、Python 等都可以通过 C ABI 互操作。
2. 编译为动态链接库（Shared Library）
Rust/C/C++ 等语言的函数被编译为共享库（.so, .dll, .dylib），Python 可以动态加载这个库并调用其导出函数。
3. 数据表示与传递
FFI 能直接传递的只有基础类型（如 int、float、指针）。
对于复杂数据结构（如字符串、数组、结构体）：
    要么手动做序列化/反序列化；
    要么双方约定内存布局和所有权。
PyO3 这类工具则自动封装了这些逻辑，开发者只需写函数签名。

## Pyo3
PyO3 是基于 Rust 宏和 CPython API 的高级封装，它用 Rust 安全系统包装了 Python 的不安全底层接口，使得你可以像写普通 Rust 函数一样，编写可以被 Python 直接调用的高性能扩展模块。
1. 基于 CPython API 做封装
Python 提供了官方 C API（即 CPython API）用于实现扩展模块。

PyO3 底层通过 unsafe 调用这些 C API 实现对 Python 对象的操作。

使用 Rust 安全封装（RAII，生命周期，类型系统）包装这些不安全操作。

2. Rust 宏系统封装导出逻辑
PyO3 提供宏来自动处理绑定逻辑：
#[pyfunction]：把 Rust 函数暴露为 Python 函数。

#[pyclass]：把 Rust 结构体暴露为 Python 类。

#[pymodule]：指定 Python 模块的初始化逻辑。

wrap_pyfunction!、wrap_pymodule!：注册函数到模块。

主要关心类型系统，python和rust对象如何映射转换的
PyO3 的类型转换通过 Rust trait 系统安全地桥接 Python 动态类型和 Rust 静态类型，虽强大但在跨语言时仍有 GIL、反射、内存复制等开销，适合用于低频或批量高效数据处理，不建议用于极高频精度要求的循环内部。