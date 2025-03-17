# BrowserTools MCP for Zed

This extension integrates BrowserTools MCP (Model Context Protocol) into Zed editor, providing browser monitoring and interaction capabilities through Zed's assistant system.

## Features

- **Browser Capture**: Capture screenshots, console logs, and DOM elements from your current browser page
- **Browser Audit**: Run various audits including accessibility, performance, SEO, and best practices
- **Browser Debug**: Start a comprehensive browser debugging session

## Prerequisites

- [Zed Editor](https://zed.dev/)
- [Rust](https://rustup.rs/) installed via rustup
- [Node.js](https://nodejs.org/) (v14 or higher)
- [BrowserTools Chrome Extension](https://github.com/AgentDeskAI/browser-tools-mcp/releases/download/v1.2.0/BrowserTools-1.2.0-extension.zip) installed in your Chrome browser

## Installation

### Option 1: Install as a Development Extension

1. Clone this repository:

   ```
   git clone https://github.com/mirageN1349/browser-tools-context-server
   cd browser-tools-context-server
   ```

2. Open Zed Editor

3. Navigate to Extensions

4. Click "Install Dev Extension" and select the directory where you cloned this repository

### Option 2: Install from Zed Extensions Registry (once published) Coming Soon

1. Open Zed Editor
2. Navigate to Extensions
3. Search for "BrowserTools Context Server"
4. Click "Install"

## Configuration

Add the following to your Zed settings.json:

```json
{
  "context_servers": {
    "browser-tools-context-server": {
      "settings": {
        "port": 3025,
        "host": "127.0.0.1"
      }
    }
  }
}
```

You can customize the port and host if needed.

## Setup Browser Extension

1. Download the [BrowserTools Chrome Extension](https://github.com/AgentDeskAI/browser-tools-mcp/)
2. Install in Chrome by navigating to chrome://extensions, enabling "Developer mode", and clicking "Load unpacked"
3. Select the unzipped extension folder

## Usage

After installing the extension and the Chrome extension:

1. Open Chrome with the BrowserTools extension enabled
2. Open Zed and ensure the BrowserTools Context Server extension is activated
3. Run the following command in your terminal to start the server manually:
   ```
   npx @agentdeskai/browser-tools-server@1.2.0
   ```
4. Open the Assistant in Zed
5. Use any of the following slash commands:

### Available Slash Commands

- `/browser-capture screenshot` - Take a screenshot of the current page
- `/browser-capture logs` - View browser console logs
- `/browser-capture errors` - View browser console errors
- `/browser-capture network` - View browser network logs
- `/browser-capture network-errors` - View browser network errors
- `/browser-capture clear` - Clear logged data
- `/browser-capture element` - Get information about a selected DOM element

- `/browser-audit accessibility` - Run an accessibility audit
- `/browser-audit performance` - Run a performance audit
- `/browser-audit seo` - Run an SEO audit
- `/browser-audit best-practices` - Check best practices
- `/browser-audit nextjs` - Run NextJS-specific audit (if applicable)
- `/browser-audit all` - Run all available audits

- `/browser-debug start` - Start comprehensive debugger mode

## How It Works

This extension:

1. Integrates with Zed's context server system to launch the BrowserTools MCP server
2. Automatically installs required npm packages (@agentdeskai/browser-tools-mcp and @agentdeskai/browser-tools-server)
3. Provides slash commands that communicate with the MCP server
4. The MCP server communicates with the Chrome extension to capture and analyze browser data

## Troubleshooting

- Make sure the Chrome extension is installed and enabled
- Check that your browser is running and the extension is active (you should see the BrowserTools icon in Chrome)
- Verify that the ports in your configuration match those used by the Chrome extension
- If you encounter issues, try restarting both Chrome and Zed

## Development

To modify this extension:

1. Make changes to the source code
2. Reinstall as a dev extension in Zed
3. Test your changes

## License

MIT
