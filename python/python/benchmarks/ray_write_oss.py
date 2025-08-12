import os
os.environ["DATAHUB_TOKEN"] = "eyJhbGciOiJSUzI1NiIsImtpZCI6ImM1Nzk4ZTY1LTdhN2YtNGQ0MC1iZjZlLWYxNGFiYTcwODY1MyJ9.eyJpc3MiOiJodHRwczovL2lhbS54YW1pbmltLmNvbSIsInN1YiI6InUtbDhlcnhkYmJ4ciIsImF1ZCI6WyI0eWQwM21wZTBndmJ3b3k5OXJjcCJdLCJleHAiOjE3NTQ3MDYzMzAsImlhdCI6MTc1MzQxMDMzMCwiYXRfaGFzaCI6IlZVQ0FkSmdxWEZ6UnJMMHllM2pFdkEiLCJjX2hhc2giOiJES0ZXN251LTdZLUJpM3B5QWtJdXdBIiwiZW1haWwiOiJsYWlmdUBtaW5pbWF4aS5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwibmFtZSI6IuadpeemjyIsInByZWZlcnJlZF91c2VybmFtZSI6IuadpeemjyJ9.rj5_6UQl9EugWfJCgDZO-xgg6oznC_b3K7B54qlt8VmF1vdyOpHqE-FG5-Ctq1uf2M3on1UYidaorYH57Lw2Q_GcvUDi-DFhuEisiE6y0A9hTgRf8uzA8fKvhZYYxtLcYszbPfarW331hCLOyVd_Z8UI7dMklf__txBYYB4mC8ZzJiffS7oGn8rrYxkw_uAGO3Q62q9COeHuV9kqXI3w2MgqDYawwQxEWmvFf37OjCK_iCdOTDYIMAVxwsqwY5PXhs_O6Rz5XsofdnhXRWExckxPTFy_WMpBCsPLLMajFluqLorZcmBHYqrmpTnBJGwOdRopzl4e6av20etOQcWbVA"
import uuid
import lance
import pyarrow as pa
import ray

from fusionflowkit.datahub_client import (
    fetch_secrets,
)

def get_storage_options():
    secrets = fetch_secrets()
    return {
        "oss_endpoint": secrets.oss_endpoint,
        "oss_access_key_id": secrets.oss_access_key,
        "oss_secret_access_key": secrets.oss_secret_key,
        "oss_region": secrets.oss_region,
    }

if __name__ == "__main__":
    storage_options = get_storage_options()
    from lance.ray.sink import write_lance

    table_name = uuid.uuid4().hex
    s3_bucket = "jfs-global-alishprod01"

    table_dir = f"oss://{s3_bucket}/lance_test/{table_name}"
    print(f"Saving table to {table_dir}")

    schema = pa.schema([pa.field("id", pa.int64()), pa.field("str", pa.string())])
    ds = ray.data.range(10).map(lambda x: {"id": x["id"], "str": f"str-{x['id']}"})
    write_lance(ds, table_dir, schema=schema, storage_options=storage_options)