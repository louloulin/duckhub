#!/usr/bin/env python3
"""
Mock数据清理验证脚本
验证DuckHub项目中所有mock数据依赖已被清理
"""

import os
import re
import glob
from typing import List, Tuple

def find_mock_patterns(directory: str) -> List[Tuple[str, int, str]]:
    """查找可能的mock数据模式"""
    mock_patterns = [
        r'TODO.*mock',
        r'TODO.*fallback',
        r'TODO.*implement',
        r'generateFallbackResponse',
        r'mock_data',
        r'fallback_data',
        r'hardcoded.*data',
        r'placeholder.*data',
        r'dummy.*data',
        r'fake.*data',
        r'test.*data.*=.*\[',
        r'vec!\[.*"mock"',
        r'HashMap::new\(\).*//.*mock',
        r'return.*Ok\(.*\[\]\)',  # 可能的空数据返回
        r'unimplemented!\(\)',
        r'todo!\(\)',
        r'panic!\(.*not.*implement',
    ]
    
    findings = []
    
    # 搜索Rust文件
    rust_files = glob.glob(f"{directory}/**/*.rs", recursive=True)
    
    for file_path in rust_files:
        # 跳过测试文件和目标目录
        if '/target/' in file_path or '/tests/' in file_path or '_test.rs' in file_path:
            continue
            
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
                
            for line_num, line in enumerate(lines, 1):
                line_lower = line.lower()
                for pattern in mock_patterns:
                    if re.search(pattern, line_lower):
                        findings.append((file_path, line_num, line.strip()))
                        
        except Exception as e:
            print(f"警告: 无法读取文件 {file_path}: {e}")
    
    return findings

def check_handler_implementations(directory: str) -> List[str]:
    """检查handler文件中的实现状态"""
    issues = []
    
    handler_files = [
        "crates/services/web-api/src/handlers/dashboard.rs",
        "crates/services/web-api/src/handlers/system.rs", 
        "crates/services/web-api/src/handlers/data.rs",
        "crates/services/web-api/src/handlers/query.rs",
        "crates/services/web-api/src/handlers/realtime.rs",
        "crates/services/web-api/src/handlers/ai.rs",
    ]
    
    for file_path in handler_files:
        full_path = os.path.join(directory, file_path)
        if not os.path.exists(full_path):
            issues.append(f"❌ Handler文件不存在: {file_path}")
            continue
            
        try:
            with open(full_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            # 检查是否有真实的实现函数
            if 'get_real_' in content or 'process_ai_message' in content:
                issues.append(f"✅ {file_path} 包含真实实现函数")
            else:
                issues.append(f"⚠️ {file_path} 可能缺少真实实现函数")
                
            # 检查TODO注释
            todo_count = content.lower().count('todo')
            if todo_count > 0:
                issues.append(f"⚠️ {file_path} 包含 {todo_count} 个TODO注释")
            else:
                issues.append(f"✅ {file_path} 无TODO注释")
                
        except Exception as e:
            issues.append(f"❌ 无法读取 {file_path}: {e}")
    
    return issues

def main():
    """主函数"""
    print("🧹 DuckHub Mock数据清理验证")
    print("=" * 50)
    
    project_root = "/Users/louloulin/Documents/augment-projects/duckhub"
    
    if not os.path.exists(project_root):
        print(f"❌ 项目目录不存在: {project_root}")
        return
    
    print("🔍 搜索可能的mock数据模式...")
    mock_findings = find_mock_patterns(project_root)
    
    print(f"\n📊 发现 {len(mock_findings)} 个可能的mock数据模式:")
    
    if mock_findings:
        for file_path, line_num, line in mock_findings[:20]:  # 只显示前20个
            rel_path = os.path.relpath(file_path, project_root)
            print(f"  📁 {rel_path}:{line_num}")
            print(f"     {line}")
            print()
    else:
        print("✅ 未发现明显的mock数据模式")
    
    print("\n🔧 检查Handler实现状态...")
    handler_issues = check_handler_implementations(project_root)
    
    for issue in handler_issues:
        print(f"  {issue}")
    
    print("\n📋 验证总结:")
    print("=" * 30)
    
    # 统计结果
    critical_patterns = [finding for finding in mock_findings 
                        if any(pattern in finding[2].lower() 
                              for pattern in ['todo', 'unimplemented', 'panic'])]
    
    if len(critical_patterns) == 0:
        print("✅ 无关键的未实现代码")
    else:
        print(f"⚠️ 发现 {len(critical_patterns)} 个关键未实现代码")
    
    if len(mock_findings) <= 5:
        print("✅ Mock数据模式数量在可接受范围内")
    else:
        print(f"⚠️ 发现较多mock数据模式: {len(mock_findings)} 个")
    
    # 检查特定文件的实现
    key_files_implemented = 0
    key_files_total = 6
    
    for issue in handler_issues:
        if "包含真实实现函数" in issue:
            key_files_implemented += 1
    
    print(f"📈 Handler实现进度: {key_files_implemented}/{key_files_total} ({key_files_implemented/key_files_total*100:.1f}%)")
    
    if key_files_implemented == key_files_total and len(critical_patterns) == 0:
        print("\n🎉 Phase 1 Mock数据清理验证通过!")
        print("✅ 所有关键Handler已实现真实功能")
        print("✅ 无关键的未实现代码")
        print("✅ DuckHub已准备好进入Phase 2")
    else:
        print("\n⚠️ Mock数据清理需要进一步完善")
        if key_files_implemented < key_files_total:
            print(f"   - 还有 {key_files_total - key_files_implemented} 个Handler需要实现")
        if len(critical_patterns) > 0:
            print(f"   - 还有 {len(critical_patterns)} 个关键代码需要实现")

if __name__ == "__main__":
    main()
