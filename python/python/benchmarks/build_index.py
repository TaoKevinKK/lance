import os
os.environ['LANCE_LOG']='DEBUG'
os.environ['LANCE_FTS_NUM_SHARDS']='1'
import shutil

import lance
import numpy as np
import pandas as pd
import pyarrow as pa

df = pd.read_csv("frankenstein_random_fragments.csv")
result_path = "/tmp/lance/dataset/test.lance"
shutil.rmtree(result_path, ignore_errors=True)

ds = lance.write_dataset(df, result_path)
frags = ds.get_fragments()
ds.create_scalar_index("text", index_type="INVERTED", with_position=False, fragment_ids=[0, 1])
