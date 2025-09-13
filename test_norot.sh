#!/bin/bash

# NoRot Testing Script
echo "🛡️ NoRot Content Filter Testing Script"
echo "======================================="
echo

# Check if NoRot is running
echo "1. Checking if NoRot is running..."
if curl -s http://localhost:8080 > /dev/null; then
    echo "✅ NoRot is running on http://localhost:8080"
else
    echo "❌ NoRot is not running. Please start it with: cargo run"
    exit 1
fi

echo

# Test the dashboard
echo "2. Testing dashboard access..."
if curl -s http://localhost:8080 | grep -q "NoRot Dashboard"; then
    echo "✅ Dashboard is accessible"
else
    echo "❌ Dashboard is not accessible"
fi

echo

# Test API endpoints
echo "3. Testing API endpoints..."
echo "   - Stats API..."
if curl -s http://localhost:8080/api/stats | grep -q "total_requests"; then
    echo "   ✅ Stats API working"
else
    echo "   ❌ Stats API not working"
fi

echo "   - Recent content API..."
if curl -s http://localhost:8080/api/recent > /dev/null; then
    echo "   ✅ Recent content API working"
else
    echo "   ❌ Recent content API not working"
fi

echo "   - Config API..."
if curl -s http://localhost:8080/api/config | grep -q "filters"; then
    echo "   ✅ Config API working"
else
    echo "   ❌ Config API not working"
fi

echo

# Test content filtering
echo "4. Testing content filtering..."
echo "   - Testing clickbait content (should be blocked)..."
RESPONSE=$(curl -s "http://localhost:8080/proxy?url=http://localhost:9000/test_content.html")
if echo "$RESPONSE" | grep -q "Content Blocked"; then
    echo "   ✅ Clickbait content successfully blocked"
else
    echo "   ❌ Clickbait content was not blocked"
fi

echo "   - Testing educational content (should be allowed)..."
RESPONSE=$(curl -s "http://localhost:8080/proxy?url=http://localhost:9000/educational_content.html")
if echo "$RESPONSE" | grep -q "Learn Python Programming"; then
    echo "   ✅ Educational content successfully allowed"
else
    echo "   ❌ Educational content was not allowed"
fi

echo

# Test bypass functionality
echo "5. Testing bypass functionality..."
RESPONSE=$(curl -s "http://localhost:8080/proxy?url=http://localhost:9000/test_content.html&norot_bypass=1")
if echo "$RESPONSE" | grep -q "You Won't Believe This Shocking Viral Content"; then
    echo "   ✅ Bypass functionality working"
else
    echo "   ❌ Bypass functionality not working"
fi

echo

echo "🎉 NoRot testing complete!"
echo
echo "📊 View the dashboard at: http://localhost:8080"
echo "🔗 Test content filtering: http://localhost:8080/proxy?url=http://localhost:9000/test_content.html"
echo "📚 Test educational content: http://localhost:8080/proxy?url=http://localhost:9000/educational_content.html"