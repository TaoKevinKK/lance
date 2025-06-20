import pandas as pd
import re

def parse_textfile_to_csv(input_file_path: str, output_csv_path: str):
    """
    读取一个文本文件，文件中每行格式为：行号 \t 文本
    将其保存为 CSV 文件，包含两列：line_id, text
    """
    data = []

    with open(input_file_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            match = re.match(r"^(\d+)\s+(.*)$", line)
            if match:
                line_id = int(match.group(1))
                text = match.group(2).strip()
                data.append((line_id, text))
            else:
                print(f"跳过无法解析的行: {line}")

    df = pd.DataFrame(data, columns=["line_id", "text"])
    df.to_csv(output_csv_path, index=False)
    print(f"已保存 CSV 至: {output_csv_path}")



parse_textfile_to_csv("mm_dataset.txt", "lance_dataset.csv")

