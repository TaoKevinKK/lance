# BlobFile 与 LanceBlobFile

## Python 端

* `BlobFile` 类继承自 `io.RawIOBase`
* 内部成员变量为 `LanceBlobFile`（通过 PyO3 实现的 Rust 绑定）

---

## Rust 端结构

```rust
pub struct LanceBlobFile {
    inner: Arc<InnerBlobFile>,
}
```

### ✅ 使用 Arc 的目的

* **多线程共享**：
  `Arc`（Atomic Reference Counted）允许多个线程安全地共享同一个 `InnerBlobFile` 实例。

* **避免数据复制**：
  所有 `LanceBlobFile` 实例共享相同底层资源，节省内存和构造开销。

* **自动生命周期管理**：
  最后一个 `Arc` 被销毁时，`InnerBlobFile` 会自动析构释放资源。

* **轻量级 clone**：
  `Arc::clone()` 仅增加引用计数，不会复制数据本体。

---

## 示例方法解析

```rust
pub fn seek(&self, py: Python<'_>, position: u64) -> PyResult<()> {
    let inner = self.inner.clone();
    RT.block_on(Some(py), inner.seek(position))?.infer_error()
}
```

### 方法解释：

1. **`self.inner.clone()`**

   * 通过 `Arc::clone` 获取 `InnerBlobFile` 的共享引用。
   * 避免数据复制，线程安全地传递引用给异步操作。

2. **`RT.block_on(Some(py), inner.seek(position))`**

   * `RT` 是一个全局的 Tokio 异步运行时（例如 `tokio::runtime::Runtime`）。
   * `block_on` 会阻塞当前线程，直到异步操作完成。
   * `Some(py)` 提供 PyO3 的 GIL token，使异步操作在 Python 环境下安全执行。
   * `inner.seek(position)` 是一个 `async fn`，表示执行跳转文件偏移的异步操作。

3. **`.infer_error()`**

   * 用于将 Rust 层的错误转换为 Python 可识别的异常，符合 PyO3 的错误处理规范。

### 总结

该方法通过克隆 `Arc` 实例，在持有 Python GIL 的前提下，安全地将异步 Rust 操作与 Python 环境集成，是一种常见的 Rust + Python 异步桥接模式。

## BlobFile(rust)实现

```rust
pub struct BlobFile {
    dataset: Arc<Dataset>,
    reader: Arc<Mutex<ReaderState>>,
    // 数据文件的路径
    data_file: Path,
    // 在文件中的起始位置
    position: u64,
    size: u64,
}
```
创建一个新的BlobFile：
1. 通过行地址获取fragment id（行地址实现高32位fragment id， 低32位偏移量的编码）
2. 通过fragment获取对应的数据文件（可指定列名，只要对应的字段数据文件）

设计特点：
1. 惰性读取：文件不会立即打开，只在需要时才打开
2. 状态管理：通过 ReaderState 管理文件打开/关闭状态
3. 线程安全：使用 Arc<Mutex<>> 支持多线程访问
4. 精确定位：知道 blob 在文件中的确切位置和大小
5. 数据集关联：保持对原数据集的引用，便于访问元数据

### take_blobs的核心逻辑
1. 验证列是否为blob列和数据类型
2. 因为持有dataset的引用，根据行id，获取行地址信息
3. 解析行地址，大小，偏移量，创建一个blobfile对象返回，内部有一个reader（需要的时候再读）