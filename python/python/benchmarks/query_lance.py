import os
os.environ['LANCE_LOG']='DEBUG'
import lance
uri = "/Users/laifu/dataset/test.lance"
ds = lance.dataset(uri)

query = """
Ruined castles hanging on the banks of the same opinion, and that confirms me.
"""
table  = ds.scanner(columns=["text"], full_text_query=query).to_table()
print(table)
