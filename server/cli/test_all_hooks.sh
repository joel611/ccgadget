#!/bin/bash

# Test script for all hook event types using compiled binary
set -e

echo "Testing CCGadget CLI with compiled binary"
echo "========================================"

cd "$(dirname "$0")"

# Check if release binary exists, build if not
if [ ! -f "target/release/ccgadget" ]; then
    echo "🔨 Release binary not found, building..."
    ./build.sh
fi

BINARY="./target/release/ccgadget"

echo -e "\n🧪 Testing compiled binary: $BINARY"
echo "Binary version:"
$BINARY --help | head -3

echo -e "\n1. Testing UserPromptSubmit hook:"
cat test_data/user_prompt_submit.json | $BINARY trigger

echo -e "\n2. Testing Notification hook:"
cat test_data/notification.json | $BINARY trigger

echo -e "\n3. Testing PreToolUse hook:"
cat test_data/pre_tool_use.json | $BINARY trigger

echo -e "\n4. Testing PostToolUse hook:"
cat test_data/post_tool_use.json | $BINARY trigger

echo -e "\n5. Testing pair command help:"
$BINARY pair --help

echo -e "\n✅ All tests completed successfully!"
echo "📋 Check logs at: ~/.ccgadget/logs/trigger-$(date +%Y-%m-%d).log"