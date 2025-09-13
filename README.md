# NoRot - Social Media Content Filter

> 🛡️ **Protecting you from doomscrolling with intelligent content filtering**

NoRot is a Rust-based proxy server that intelligently filters or throttles low-value social media content to help preserve your time and mental well-being. Using AI/ML classification models, it distinguishes between valuable educational content and mindless "brainrot" content, allowing you to consume social media more intentionally.

![NoRot Dashboard](https://github.com/user-attachments/assets/ea16a375-6b35-47d2-babf-e953e8621304)

## ✨ Features

### 🎯 Intelligent Content Classification
- **Smart AI Detection**: Uses machine learning to classify content into categories
  - 🧠 **Educational**: Tutorials, guides, learning content → **ALLOWED**
  - 📰 **Informative**: News, research, analysis → **ALLOWED** 
  - 🎣 **Clickbait**: "You won't believe...", "Shocking..." → **BLOCKED**
  - 📱 **Mindless Scrolling**: Endless social media feeds → **THROTTLED**

### 🚫 Flexible Content Actions
- **Block**: Completely blocks harmful content with a beautiful warning page
- **Throttle**: Adds delays to reduce impulse browsing
- **Warning**: Injects warning banners into questionable content
- **Allow**: Permits valuable and educational content

### 📊 Beautiful Web Dashboard
- **Real-time Statistics**: Track blocked, throttled, and allowed requests
- **Activity Monitoring**: See detailed classification results with confidence scores
- **Filter Configuration**: Toggle-based UI for customizing rules
- **Dark Theme**: Modern, eye-friendly interface

### 🔧 Configurable & Extensible
- **TOML Configuration**: Easy-to-edit configuration files
- **Multiple AI Models**: Mock, OpenAI, or local model support
- **Custom Rules**: Define your own filtering rules and thresholds
- **Database Logging**: SQLite database for statistics and history

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

### Installation & Running

```bash
# Clone the repository
git clone https://github.com/xavilien/norot.git
cd norot

# Build and run
cargo run

# The server will start on http://localhost:8080
```

### Testing the Filter

1. **Visit the Dashboard**: http://localhost:8080
2. **Test Clickbait Content**: http://localhost:8080/proxy?url=http://localhost:9000/test_content.html
3. **Test Educational Content**: http://localhost:8080/proxy?url=http://localhost:9000/educational_content.html

Run the included test script:
```bash
./test_norot.sh
```

## 🔧 Usage

### Browser Proxy Setup
Configure your browser to use NoRot as a proxy:
- **Proxy Server**: `127.0.0.1:8080`
- **Type**: HTTP Proxy

### Direct URL Filtering
Test content filtering by visiting:
```
http://localhost:8080/proxy?url=https://example.com
```

### Bypass Protection
When content is blocked, you can choose to "Proceed Anyway" - use this feature mindfully!

## 📁 Project Structure

```
norot/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration management
│   ├── proxy.rs          # HTTP proxy server
│   ├── classifier.rs     # Content classification engine
│   ├── filter.rs         # Content filtering logic
│   ├── db.rs            # Database operations
│   └── ui.rs            # Web UI handlers
├── templates/
│   └── dashboard.html    # Dashboard template
├── static/
│   ├── style.css        # Dashboard styles
│   └── script.js        # Dashboard JavaScript
├── data/
│   └── norot.db         # SQLite database
├── config.toml          # Configuration file
└── test_norot.sh        # Test script
```

## ⚙️ Configuration

Edit `config.toml` to customize filtering behavior:

```toml
[filters]
enabled = true
notification_threshold = 0.8

[filters.rules.brainrot]
enabled = true
score_threshold = 0.7
action = "Block"
categories = ["clickbait", "mindless_scrolling"]

[filters.rules.educational]
enabled = true
score_threshold = 0.6
action = "Allow"
categories = ["learning", "tutorial", "informative"]

[classifier]
enabled = true
model_type = "Mock"  # "Mock", "OpenAI", or "Local"
confidence_threshold = 0.6
```

## 🧠 How Content Classification Works

NoRot analyzes content using multiple signals:

1. **URL Analysis**: Checks domain and path patterns
2. **Title & Description**: Scans meta tags for keywords
3. **Content Keywords**: Analyzes text content for patterns
4. **Social Media Patterns**: Identifies platform-specific behavior

### Example Classifications:

**Educational Content** (Allowed):
- "Learn Python Programming Tutorial"
- "How to Build REST APIs"
- "Machine Learning Guide"

**Clickbait Content** (Blocked):
- "You Won't Believe What Happens Next!"
- "This One Weird Trick..."
- "Doctors Hate Him..."

**Social Media Patterns** (Throttled):
- Instagram stories and reels
- TikTok endless scroll feeds
- Twitter/X recommendation feeds

## 🛡️ Content Blocking Example

When harmful content is detected, users see a thoughtful blocking page:

![Content Blocked](https://github.com/user-attachments/assets/0652948a-5e5d-41db-9d42-0cb8b586c043)

## 🔮 Future Enhancements

- 🌐 **Browser Extension**: Easy installation without proxy setup
- 🤖 **Real AI Integration**: OpenAI GPT, local LLM models
- 📱 **Mobile App**: iOS and Android applications
- 👥 **User Accounts**: Personal settings and statistics
- 🔔 **Push Notifications**: Real-time blocking alerts
- 📈 **Advanced Analytics**: Detailed usage patterns and insights
- 🎯 **Custom Categories**: User-defined content categories
- 🌍 **Multi-language Support**: International content filtering

## 🤝 Contributing

We welcome contributions! Areas where you can help:

- 🧠 **AI Model Integration**: Add support for more classification models
- 🎨 **UI/UX Improvements**: Enhance the dashboard and blocking pages
- 🔧 **Configuration**: Add more customization options
- 📱 **Platform Support**: Mobile apps, browser extensions
- 🧪 **Testing**: Improve test coverage and real-world testing
- 📚 **Documentation**: Help others understand and use NoRot

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with ❤️ in Rust
- Powered by [Axum](https://github.com/tokio-rs/axum) web framework
- UI inspired by modern design principles
- Thanks to the Rust community for excellent crates and documentation

---

**Remember**: The goal isn't to completely block social media, but to help you consume it more intentionally. Use the bypass feature when you genuinely want to access blocked content, but be mindful of your habits! 🧘‍♀️
