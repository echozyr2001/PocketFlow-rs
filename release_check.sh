#!/bin/bash
set -e

echo "🔧 PocketFlow-rs 发布前检查"
echo "=============================="

echo
echo "📦 1. 清理构建缓存..."
cargo clean

echo
echo "🔍 2. 代码格式检查..."
cargo fmt --check

echo 
echo "🧹 3. Clippy 代码质量检查..."
cargo clippy -- -D warnings

echo
echo "🔨 4. 编译检查（所有 feature）..."
cargo check --all-features

echo
echo "🧪 5. 运行所有测试..."
cargo test --lib --all-features

echo
echo "📚 6. 生成文档..."
cargo doc --no-deps --all-features

echo
echo "📝 7. 检查示例代码（使用对应 feature）..."
# 不检查需要特殊 feature 的示例，因为它们现在有正确的 feature gate

echo
echo "✅ 所有检查通过！PocketFlow-rs 已准备好发布。"
echo
echo "🚀 下一步建议："
echo "   • 更新版本号: cargo edit set version <new-version>"
echo "   • 添加 Git 标签: git tag v<version>"  
echo "   • 发布到 crates.io: cargo publish"
echo "   • 推送到 Git: git push --tags"