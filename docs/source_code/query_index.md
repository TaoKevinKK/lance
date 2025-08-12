## dataset.to_table
to_table 是 多功能数据读取接口，统一处理列选择、条件过滤、向量搜索、全文搜索和批量读取，返回高性能的列式数据结构.

```python
class LanceScanner(pa.dataset.Scanner):
    def __init__(self, scanner: _Scanner, dataset: LanceDataset):
        # 底层数据扫描对象，to_pyarrow
        self._scanner = scanner
        # 数据集实例，提供了表结构和元数据
        self._ds = dataset

    def to_table(self) -> pa.Table:
        """
        Read the data into memory and return a pyarrow Table.
        """
        return self.to_reader().read_all()

```

方法解析
1. to_reader()
将扫描器对象转换为 pa.RecordBatchReader。

底层是scanner调用的 to_pyarrow方法，内部执行逻辑：
a. 创建LanceScanner，作为LanceReader的成员变量，确定执行计划
b. LanceReader，作为RecordBatchReader的成员变量
c. 在LanceReader init时，LanceScanner 调用了异步函数（try_into_stream）

2. read_all()
将整个数据扫描结果一次性加载到内存中。
返回 pyarrow.Table

### LanceScanner
是lance读数据的核心对象，确定数据或者索引的扫描执行计划。
触发读数据操作的入口是方法：try_into_stream （scanner.rs）


```rust
#[instrument(skip_all)]
pub fn try_into_stream(&self) -> BoxFuture<Result<DatasetRecordBatchStream>> {
    // Future intentionally boxed here to avoid large futures on the stack
    // 异步块开始，async move 表示闭包捕获变量的所有权。
    async move {
        // 1.创建执行计划
        let plan = self.create_plan().await?;
        // 3. 执行扫描计划，构造数据流对象
        Ok(DatasetRecordBatchStream::new(execute_plan(
            plan,
            // 2. 构造执行参数，batch_size等
            LanceExecutionOptions {
                batch_size: self.batch_size,
                execution_stats_callback: self.scan_stats_callback.clone(),
                ..Default::default()
            },
        )?))
    }
    .boxed()
    // boxed用法：将整个async move变成堆上的trait对象，可防止异步计算返回过大或不可控的结果
}
```

#### create_plan 创建执行计划
重点可关注索引读取的执行计划（fts_search_source）
执行计划-优化器（optimizer.rs）
    将相邻的TakeExec合并，减少多次取列
    去除无效的prokection
#### execute_plan 执行

