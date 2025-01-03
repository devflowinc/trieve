# trieve-mcp-server

A Model Context Protocol (MCP) server that provides agentic tools for interacting with the Trieve API. This server enables AI agents to search and interact with Trieve datasets through a standardized interface.

## Features

- Search across Trieve datasets using semantic search
- List and access dataset information
- Support for both environment variables and command-line arguments
- Built with TypeScript for type safety and better developer experience

## Installation

```bash
npm install trieve-mcp-server
```

## Configuration

Copy the `.env.dist` file to `.env` and fill in your Trieve credentials:

```bash
cp .env.dist .env
```

Required environment variables:
- `TRIEVE_API_KEY`: Your Trieve API key from dashboard.trieve.ai
- `TRIEVE_ORGANIZATION_ID`: Your Trieve organization ID from dashboard.trieve.ai

Optional environment variables:
- `TRIEVE_DATASET_ID`: Specific dataset ID to use (if not provided via CLI)

Command-line arguments (override environment variables):
```bash
trieve-mcp-server --api-key <your-api-key> --org-id <your-org-id> [--dataset-id <dataset-id>]
```

## Usage

### Starting the Server

```bash
trieve-mcp-server
```

### Available Tools

#### search
Search through a specified Trieve dataset.

Parameters:
- `query` (string): The search query
- `datasetId` (string): ID of the dataset to search in
- `searchType` (string, optional): "semantic" (default), "fulltext", "hybrid", or "bm25"
- `filters` (object, optional): Advanced filtering options
- `highlightOptions` (object, optional): Customize result highlighting
- `page` (number, optional): Page number, default 1
- `pageSize` (number, optional): Results per page, default 10

Example:
```json
{
  "query": "example search query",
  "datasetId": "your-dataset-id",
  "searchType": "semantic",
  "page": 1,
  "pageSize": 10
}
```

### Available Resources

The server exposes Trieve datasets as resources with the following URI format:
- `trieve://datasets/{dataset-id}`

## Usage with Claude Desktop

The Trieve MCP Server supports MCP integration with [Claude Desktop](https://modelcontextprotocol.io/quickstart/user). Place the following in your Claude Desktop's `claude_desktop_config.json`.

```json
{
  "mcpServers": {
    "trieve-mcp-server": {
      "command": "npx",
      "args": ["trieve-mcp-server@latest"],
      "env": {
        "TRIEVE_API_KEY": "$TRIEVE_API_KEY",
        "TRIEVE_ORGANIZATION_ID": "$TRIEVE_ORGANIZATION_ID",
        "TRIEVE_DATASET_ID": "$TRIEVE_DATASET_ID"
      }
    }
  }
}
```

Note: Instead of environment variables, `--api-key`, `--org-id`, and `--dataset-id` can be used as command-line arguments.

Once Claude Desktop starts, attachments will be available that correspond to the [datasets available to the Trieve organization](https://docs.trieve.ai/guides/create-organizations-and-dataset). These can be used to select a dataset. After that, begin chatting with Claude and ask for information about the dataset. Claude will use search as needed in order to filter and break down queries, and may make multiple queries depending on your task.

## Development

### Setup

1. Clone the repository
2. Install dependencies:
```bash
npm install
```
3. Copy `.env.dist` to `.env` and configure your credentials
4. Build the project:
```bash
npm run build
```

### Scripts

- `npm run build`: Build the TypeScript project
- `npm run watch`: Watch for changes and rebuild
- `npm run test`: Run tests
- `npm run inspector`: Run the MCP inspector for debugging

## License

MIT
