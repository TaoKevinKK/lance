## 
　一个Parquet文件: 一个header,一个或多个block块组成，以一个footer结尾。
header中只包含一个4个字节的数字PAR1用来识别整个Parquet文件格式。
文件中所有的metadata都存在于footer中。footer中的metadata包含了格式的版本信息，schema信息、key-value paris以及所有block中的metadata信息。footer中最后两个字段为一个以4个字节长度的footer的metadata,以及同header中包含的一样的PAR1。

　　读取一个Parquet文件时，需要完全读取Footer的meatadata，Parquet格式文件不需要读取sync markers这样的标记分割查找，因为所有block的边界都存储于footer的metadata中.

　　在Parquet文件中，每一个block都具有一组Row group, 是由一组Column chunk组成的列数据。每一个column chunk中包含pages。每个page就包含来自于相同列的值.
　　每个page支持标准的压缩算法比如支持Snappy,gzip以及LZO压缩格式。


## 问题
### 随机访问差？
parqut的设计，更擅长批量扫描数据，不擅长单点查询或随机读取
1. Page级别无法定位单个数据，只能顺序读取
2. 需要解压对应page，找到具体的行号

https://zhuanlan.zhihu.com/p/680143641