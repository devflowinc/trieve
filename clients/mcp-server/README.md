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

Alternatively, you can provide these credentials via command-line arguments:
```bash
mcp-server-trieve --api-key <your-api-key> --org-id <your-org-id>
```

## Usage

### Starting the Server

```bash
mcp-server-trieve
```

### Available Tools

#### search
Search through a specified Trieve dataset.

Parameters:
- `query` (string): The search query
- `datasetId` (string): ID of the dataset to search in

Example:
```json
{
  "query": "example search query",
  "datasetId": "your-dataset-id"
}
```

### Available Resources

The server exposes Trieve datasets as resources with the following URI format:
- `trieve://datasets/{dataset-id}`

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
