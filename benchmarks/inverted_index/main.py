import lance
import pandas as pd
import numpy as np


def _read_text(path: str):
    data = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue  # 跳过空行
            parts = line.split("\t", 1)  # 最多分成2部分，避免文本中有tab的问题
            if len(parts) != 2:
                print(f"跳过格式错误的行: {line}")
                continue
            doc_id, text = parts
            data.append((int(doc_id), text))
    return data

def _split_fragment(df: pd.DataFrame, num_splits: int) -> list[pd.DataFrame]:
    total_rows = len(df)
    base_size = total_rows // num_splits  # 每个分片的基础行数
    remainder = total_rows % num_splits   # 不能均分的余数

    # 前 remainder 个分片多 1 行
    sizes = [base_size + 1 if i < remainder else base_size for i in range(num_splits)]

    # 累积出分割点（不包含最后一个点）
    split_points = np.cumsum(sizes)[:-1]

    # 使用 np.split 分割 DataFrame
    return np.split(df, split_points)

def table_to_tuple(tb):
    return list(zip(tb["_rowid"].to_pylist(), tb["_score"].to_pylist()))

if __name__ == '__main__':
    tmp_path = "/Users/laifu/code/lance/benchmarks/inverted_index/test_lance"
    data = _read_text("data.txt")
    df = pd.DataFrame(data, columns=["id", "text"])
    # fragments = _split_fragment(df, 4)
    # for data_fragment in fragments:
    #     ds = lance.write_dataset(data_fragment, tmp_path, mode="append")
    indices = []
    ds = lance.dataset(tmp_path)
    # frags = list(ds.get_fragments())
    # for f in frags:
    #     # we can create an inverted index distributely
    #     index = ds.create_scalar_index("text", "INVERTED", fragment_ids=[f.fragment_id])
    #     assert isinstance(index, dict)
    #     indices.append(index)
    # create_index_op = lance.LanceOperation.CreateIndex(
    #     new_indices=indices,
    #     removed_indices=[],
    # )
    # ds = lance.LanceDataset.commit(
    #     ds.uri,
    #     create_index_op,
    #     read_version=ds.version,
    # )

    text2_query_fetch = ds.to_table(
        full_text_query={"columns": ["text"], "query": "The weather was fine; it was about the middle of the lake and talked"},
        prefilter=True,
        with_row_id=True,
    )
    print(text2_query_fetch)
    print(table_to_tuple(text2_query_fetch))