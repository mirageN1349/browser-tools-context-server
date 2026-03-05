## Setup

1. Install the [BrowserTools Chrome Extension](https://github.com/AgentDeskAI/browser-tools-mcp)
2. Open Chrome with the extension enabled
3. Start the browser-tools-server:
   ```
   npx @agentdeskai/browser-tools-server@1.2.1
   ```
4. The MCP server starts automatically when the extension activates in Zed

### Requirements

- Node.js v18+
- Chrome with BrowserTools extension installed

### Troubleshooting

- Make sure Chrome is running with the BrowserTools extension enabled
- Make sure browser-tools-server is running (`npx @agentdeskai/browser-tools-server@1.2.1`)
- Check that Node.js and npx are available in your PATH
- If port 3025 is in use, configure a different port in settings
