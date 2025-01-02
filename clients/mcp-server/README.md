# Trieve MCP Server

A Model Context Protocol (MCP) server that enables using Trieve datasets as retrieval providers in MCP clients like Claude Desktop.

## Overview

This MCP server allows you to:
1. List your Trieve datasets (Projects) as available resources
2. Search within selected datasets to provide context for AI interactions
3. Use the search results to enhance AI responses with relevant context

Common use cases:
- Searching through Obsidian notes
- Retrieving information from research papers
- Finding relevant content in documentation
- Using personal knowledge bases for context

## Prerequisites

- Node.js (v16 or higher)
- A Trieve account with API access
- Access to [dashboard.trieve.ai](https://dashboard.trieve.ai)
- At least one dataset created in Trieve

## Installation

1. Install dependencies:
```bash
npm install
```

2. Copy the environment configuration:
```bash
cp .env.example .env
```

3. Configure your environment variables in `.env`:
```env
# Required: Your Trieve API credentials
TRIEVE_API_KEY=your-api-key
TRIEVE_ORGANIZATION_ID=your-org-id

# Optional: Server configuration
PORT=3000
```

You can find your API credentials at [dashboard.trieve.ai](https://dashboard.trieve.ai):
- API Key: Settings -> API Keys
- Organization ID: Settings -> Organization Details

## Building

Build the TypeScript code:
```bash
npm run build
```

For development with auto-reloading:
```bash
npm run watch
```

## Integration with Claude Desktop

1. Locate your Claude Desktop configuration file:
   - MacOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%/Claude/claude_desktop_config.json`

2. Add the server configuration to the config file:
```json
{
  "mcpServers": {
    "trieve": {
      "command": "node",
      "args": [
        "/path/to/clients/mcp-server/build/index.js"
      ]
    }
  }
}
```

3. Restart Claude Desktop

## Usage

Once configured, you can interact with your Trieve datasets through Claude Desktop:

1. List available datasets:
```
Show me my available Trieve datasets
```

2. Search within a dataset:
```
Search for 'your query' in dataset 'dataset-id'
```

3. Use search results as context:
```
Using the search results, help me understand...
```

## Available Tools

1. `get_datasets`
   - Lists all available Trieve datasets in your organization
   - No parameters required

2. `search_trieve`
   - Searches within a selected dataset
   - Parameters:
     - `dataset_id`: The ID of the dataset to search in
     - `query`: The search query

3. `calculate_sum` (test tool)
   - Adds two numbers together
   - Parameters:
     - `a`: First number
     - `b`: Second number

## Development

### Project Structure

```
clients/mcp-server/
├── src/
│   └── index.ts      # Main server implementation
├── build/            # Compiled JavaScript
├── .env.example      # Example environment configuration
├── .env             # Local environment configuration (git-ignored)
├── package.json     # Project dependencies and scripts
└── tsconfig.json    # TypeScript configuration
```

### Error Handling

The server includes comprehensive error handling for:
- Missing environment variables
- API authentication issues
- Invalid dataset IDs
- Search failures
- Invalid response formats

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Troubleshooting

1. **Server won't start**
   - Check if `.env` is properly configured
   - Verify API credentials are valid

2. **Can't see datasets**
   - Confirm `TRIEVE_ORGANIZATION_ID` is correct
   - Check if you have datasets created in Trieve

3. **Search not working**
   - Verify the dataset ID is correct
   - Ensure the dataset has content to search

## License

This project is part of the Trieve ecosystem. See the main repository for license details.

## Support

For support:
- Matrix (preferred)
- Discord
- GitHub Issues
