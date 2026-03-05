# BrowserTools MCP for Zed

This extension integrates BrowserTools MCP (Model Context Protocol) into Zed editor, providing browser monitoring and interaction capabilities through Zed's assistant system.

## Features

- **Browser Capture**: Capture screenshots, console logs, and DOM elements from your current browser page
- **Browser Audit**: Run various audits including accessibility, performance, SEO, and best practices
- **Browser Debug**: Start a comprehensive browser debugging session

## Prerequisites

- [Zed Editor](https://zed.dev/)
- [Node.js](https://nodejs.org/) (v18 or higher)
- [BrowserTools Chrome Extension](https://github.com/AgentDeskAI/browser-tools-mcp) installed in your Chrome browser

## Installation

### Option 1: Install from Zed Extensions Registry

1. Open Zed Editor
2. Navigate to Extensions
3. Search for "BrowserTools Context Server"
4. Click "Install"

### Option 2: Install as a Development Extension

1. Clone this repository:

   ```
   git clone https://github.com/mirageN1349/browser-tools-context-server
   cd browser-tools-context-server
   ```

2. Open Zed Editor

3. Navigate to Extensions

4. Click "Install Dev Extension" and select the cloned directory

## Setup

1. Install the [BrowserTools Chrome Extension](https://github.com/AgentDeskAI/browser-tools-mcp)
2. Open Chrome with the extension enabled
3. Start the browser-tools-server:
   ```
   npx @agentdeskai/browser-tools-server@1.2.1
   ```
4. The MCP server starts automatically when the extension activates in Zed

## Configuration

Add the following to your Zed settings.json to customize:

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

## Usage

After installing the extension and the Chrome extension:

1. Open Chrome with the BrowserTools extension enabled
2. Open Zed and ensure the BrowserTools Context Server extension is activated
3. Open the Assistant in Zed
4. Use any of the following slash commands:

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

This extension uses a three-component architecture:

1. **Chrome Extension** monitors browser events, console logs, network requests, and captures screenshots
2. **Browser Tools Server** (`browser-tools-server`) acts as middleware between the Chrome extension and the MCP server
3. **MCP Server** (`browser-tools-mcp`) implements the Model Context Protocol, providing standardized tools that Zed can invoke

The extension automatically launches the MCP server. The browser-tools-server must be started separately.

## Troubleshooting

- Make sure the Chrome extension is installed and enabled
- Check that your browser is running and the extension is active (you should see the BrowserTools icon in Chrome)
- Verify that Node.js and npx are available in your PATH
- If port 3025 is in use, configure a different port in settings
- If you encounter issues, try restarting both Chrome and Zed

## License

Apache-2.0
