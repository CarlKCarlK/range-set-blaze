# find all the json files below the current directory

import os
import json
from pathlib import Path


def find_json_files():
    json_files = []
    for root, dirs, files in os.walk("./target/criterion"):
        for file in files:
            if file.endswith("estimates.json"):
                root_path = Path(root)
                if root_path.name == "new":
                    json_files.append(os.path.join(root, file))
    return json_files


if __name__ == "__main__":
    json_files = find_json_files()
    # print(json_files)

    for file in json_files:
        path = Path(file)
        path = path.parts[2:5]
        with open(file) as f:
            data = json.load(f)
            try:
                time = data["mean"]["point_estimate"]
            except Exception:
                time = "missing"
            # print path and time separated by a commas
            print("\t".join(path), "\t", time)
