#!/usr/bin/env python3

import os
import re

def fix_execute_calls(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 修复 execute 方法调用
    pattern = r'\.execute\(([^,]+), \[\]\)'
    replacement = r'.execute::<&str>(\1, &[])'
    
    modified_content = re.sub(pattern, replacement, content)
    
    if content != modified_content:
        with open(file_path, 'w') as f:
            f.write(modified_content)
        print(f"Fixed execute calls in {file_path}")

def main():
    # 修复 ducklake.rs 文件
    ducklake_path = 'crates/core/database/src/ducklake.rs'
    if os.path.exists(ducklake_path):
        fix_execute_calls(ducklake_path)
    
    # 修复其他文件
    database_dir = 'crates/core/database/src'
    for filename in os.listdir(database_dir):
        if filename.endswith('.rs') and filename != 'duckdb.rs' and filename != 'mock_duckdb.rs':
            file_path = os.path.join(database_dir, filename)
            fix_execute_calls(file_path)

if __name__ == "__main__":
    main()
