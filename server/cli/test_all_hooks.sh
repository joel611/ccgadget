#!/bin/bash

# Test script for all hook event types
echo "Testing all hook event types with ccgadget trigger command"
echo "========================================================="

cd "$(dirname "$0")"

echo -e "\n1. Testing UserPromptSubmit hook:"
cat test_data/user_prompt_submit.json | cargo run -- trigger

echo -e "\n2. Testing Notification hook:"
cat test_data/notification.json | cargo run -- trigger

echo -e "\n3. Testing PreToolUse hook:"
cat test_data/pre_tool_use.json | cargo run -- trigger

echo -e "\n4. Testing PostToolUse hook:"
cat test_data/post_tool_use.json | cargo run -- trigger

echo -e "\nTest completed! Check logs at ~/.ccgadget/logs/trigger-$(date +%Y-%m-%d).log"