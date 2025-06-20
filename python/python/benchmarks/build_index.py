import os
os.environ['LANCE_LOG']='DEBUG'

import shutil

import lance
import numpy as np
import pandas as pd
import pyarrow as pa

df = pd.read_csv("lance_dataset.csv")
result_path = "/Users/laifu/dataset/test.lance"
shutil.rmtree(result_path, ignore_errors=True)

ds = lance.write_dataset(df, result_path)
ds.create_scalar_index("text", index_type="INVERTED", with_position=False)
