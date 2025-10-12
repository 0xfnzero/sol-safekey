#!/bin/bash

# Solana Bot Startup Script (Example)
# 安全启动脚本 - 密码只在内存中，不保存到环境变量或文件
#
# 使用方法:
#   ./startup-example.sh          # 编译并启动
#   ./startup-example.sh --skip-build  # 跳过编译直接启动

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════╗"
echo "║   Solana Trading Bot with sol-safekey ║"
echo "╚════════════════════════════════════════╝"
echo ""

# 检查是否跳过编译
SKIP_BUILD=false
if [ "$1" == "--skip-build" ]; then
    SKIP_BUILD=true
fi

# 检查并编译
if [ "$SKIP_BUILD" = false ]; then
    echo -e "${BLUE}📦 Building bot (release mode with solana-ops feature)...${NC}"
    echo ""

    cargo build --example bot_example --features solana-ops --release

    if [ $? -ne 0 ]; then
        echo ""
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi

    echo ""
    echo -e "${GREEN}✅ Build completed${NC}"
    echo ""
fi

# 检查编译产物
if [ ! -f "build-cache/release/examples/bot_example" ]; then
    echo -e "${RED}❌ Bot executable not found${NC}"
    echo "Expected: build-cache/release/examples/bot_example"
    exit 1
fi

# 检查 keystore 文件
if [ ! -f "./keystore.json" ]; then
    echo -e "${YELLOW}⚠️  Keystore file not found: ./keystore.json${NC}"
    echo ""
    echo "First time setup: Create your encrypted wallet"
    echo ""
    echo -n "Enter password for new wallet: "
    read -s NEW_PASSWORD
    echo ""
    echo -n "Confirm password: "
    read -s CONFIRM_PASSWORD
    echo ""

    if [ "$NEW_PASSWORD" != "$CONFIRM_PASSWORD" ]; then
        echo -e "${RED}❌ Passwords do not match${NC}"
        exit 1
    fi

    if [ -z "$NEW_PASSWORD" ]; then
        echo -e "${RED}❌ Password cannot be empty${NC}"
        exit 1
    fi

    # 创建钱包 (通过管道传递密码)
    echo "$NEW_PASSWORD" | ./build-cache/release/examples/bot_example
    NEW_PASSWORD=""
    CONFIRM_PASSWORD=""

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Wallet created successfully!${NC}"
        echo "You can now run ./startup-example.sh again to start the bot"
    else
        echo -e "${RED}❌ Failed to create wallet${NC}"
    fi
    exit 0
fi

# 安全获取密码 (不回显)
echo ""
echo -n "🔐 Enter wallet password: "
read -s WALLET_PASSWORD
echo ""

if [ -z "$WALLET_PASSWORD" ]; then
    echo -e "${RED}❌ Password cannot be empty${NC}"
    exit 1
fi

# 启动bot - 通过管道传递密码 (密码只在内存中)
echo ""
echo -e "${BLUE}🚀 Starting bot...${NC}"
echo ""

echo "$WALLET_PASSWORD" | ./build-cache/release/examples/bot_example > bot.log 2>&1
BOT_EXIT_CODE=$?

# 立即清除密码变量
WALLET_PASSWORD=""
unset WALLET_PASSWORD

# 检查退出码
if [ $BOT_EXIT_CODE -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ Bot completed successfully!${NC}"
    echo ""
    echo "📝 View full output: cat bot.log"
    echo "🔍 Wallet address: $(grep 'Address:' bot.log | tail -1 | awk '{print $NF}')"
    echo ""
else
    echo ""
    echo -e "${RED}❌ Bot failed with exit code: $BOT_EXIT_CODE${NC}"
    echo "Check bot.log for errors"
    echo ""
    exit 1
fi
