# Trieve MCP Server

A Model Context Protocol server that allows using Trieve as a retrieval provider for Claude Desktop and other MCP clients.

## Overview

This server allows you to use your Trieve datasets (Projects) as context sources in Claude Desktop. Common use cases include:
- Searching through your Obsidian notes
- Retrieving information from research papers
- Finding relevant content in your blog posts or documentation
- Any other text content you've uploaded to Trieve

## Setup

1. Create and Configure Your Trieve Dataset:
   - Go to [dashboard.trieve.ai](https://dashboard.trieve.ai)
   - Create a new dataset (Project)
   - Upload your files (e.g., Obsidian notes, PDFs, blog posts)
   - Note: You can create multiple datasets for different types of content

2. Get Your Trieve Credentials:
   - Go to Settings -> API Keys to get your API key
   - Go to Settings -> Organization Details to get your Organization ID

3. Install Dependencies:
```bash
npm install
```

4. Configure Environment:
```bash
cp .env.example .env
# Edit .env and add your:
# - TRIEVE_API_KEY
# - TRIEVE_ORGANIZATION_ID
```

5. Build the Server:
```bash
npm run build
```

## Integration with Claude Desktop

1. Configure Claude Desktop:
   - Location of config file:
     - MacOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
     - Windows: `%APPDATA%/Claude/claude_desktop_config.json`

2. Add the server configuration:
```json
{
  "mcpServers": {
    "Trieve MCP Server": {
      "command": "/absolute/path/to/build/index.js"
    }
  }
}
```

3. Using in Claude Desktop:
   - Start a new chat in Claude Desktop
   - Select "Trieve MCP Server" from the context providers
   - Choose which dataset (Project) to search against
   - Claude will now use your selected dataset as context when answering questions

## Example Use Cases

1. **Obsidian Notes Integration**:
   - Upload your Obsidian vault to a Trieve dataset
   - Claude can now reference your notes when answering questions
   - Great for personal knowledge management

2. **Research Paper Analysis**:
   - Upload research papers to a dedicated dataset
   - Ask Claude to analyze or compare papers
   - Get insights based on your research collection

3. **Documentation Search**:
   - Upload your project documentation or technical blogs
   - Claude can help explain concepts using your own documentation
   - Perfect for technical discussions about your projects

## Development

For development with auto-rebuild:
```bash
npm run watch
```

For debugging:
```bash
npm run inspector
```

## Note on Data Privacy

Your data stays within your Trieve account. The server only facilitates search and retrieval - all content remains under your control in your Trieve datasets.
