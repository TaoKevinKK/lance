
## 数据如何组织存储的？
按行水平切分，最小组织单元为Fragment；每个Fragment 下有一个或多个 .lance 数据文件（DataFile）。
一个 DataFile 可以同时存放该 Fragment 的“多个列”的列式数据与元数据。
新增列时，会给每个 Fragment 增加相应的新 DataFile（而不是重写整个 Fragment）。
同一列的数据会分布在多个 Fragment 的多个文件里（按行分块），读取时通过 Manifest/索引拼接。
删除记录存放在独立的 _deletions 文件；二级/向量索引在 _indices。