#!/bin/bash
# XORE 搜索功能演示脚本

set -e

echo "=== XORE 搜索功能演示 ==="
echo

# 检查 xore 是否可用
if ! command -v xore &> /dev/null; then
    if [ -f "./target/release/xore" ]; then
        XORE="./target/release/xore"
    else
        echo "错误: 找不到 xore 命令"
        echo "请先运行: cargo build --release"
        exit 1
    fi
else
    XORE="xore"
fi

echo "使用命令: $XORE"
echo

# 1. 标准搜索
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 标准全文搜索"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "命令: $XORE f \"error\" --index"
echo
$XORE f "error" --index
echo

# 2. 前缀搜索
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. 前缀搜索 - 搜索以 'err' 开头的词"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "命令: $XORE f \"err*\" --index"
echo
$XORE f "err*" --index
echo

# 3. 模糊搜索
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. 模糊搜索 - 容忍拼写错误"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "命令: $XORE f \"~eror\" --index"
echo
$XORE f "~eror" --index
echo

# 4. 文件类型过滤
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. 搜索特定文件类型"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "命令: $XORE f \"TODO\" --index --type rs"
echo
$XORE f "TODO" --index --type rs
echo

echo "✓ 演示完成！"
echo
echo "更多示例："
echo "  - 中文搜索: $XORE f \"错误\" --index"
echo "  - 增量监控: $XORE f \"error\" --index --watch"
echo "  - 性能测试: $XORE benchmark --suite search --data-path examples/benchmark-data/large -n 3"
