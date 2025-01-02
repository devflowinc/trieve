#!/usr/bin/env node

/**
 * Trieve MCP Server
 * 
 * This server allows using Trieve datasets as retrieval providers in MCP clients like Claude Desktop.
 * It enables users to:
 * 1. List their Trieve datasets (Projects) as available resources
 * 2. Search within selected datasets to provide context for AI interactions
 * 
 * Common use cases:
 * - Searching through Obsidian notes
 * - Retrieving information from research papers
 * - Finding relevant content in documentation
 * 
 * The server uses stdio transport for MCP communication and requires:
 * - TRIEVE_API_KEY: Your Trieve API key
 * - TRIEVE_ORGANIZATION_ID: Your Trieve organization ID
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { TrieveSDK } from 'trieve-ts-sdk';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';

// Get the directory name of the current module
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load environment variables from .env file
const result = dotenv.config({ path: path.resolve(__dirname, '../.env') });

if (result.error) {
  throw new Error(`Error loading .env file: ${result.error.message}`);
}

// Types for Trieve API responses
interface DatasetResponse {
  dataset: {
    id: string;
    name: string;
  };
  dataset_usage: {
    chunk_count: number;
  };
}

interface ChunkData {
  chunk_html?: string;
}

interface SearchChunk {
  chunk: ChunkData;
  score: number;
}

// Initialize Trieve SDK with API credentials
const trieve = new TrieveSDK({
  apiKey: process.env.TRIEVE_API_KEY!,
  baseUrl: 'https://api.trieve.ai',
});

// Create MCP server with Trieve integration
const server = new Server(
  {
    name: "Trieve MCP Server",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {}
    }
  }
);

// Define available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [{
      name: "calculate_sum",
      description: "Add two numbers together",
      inputSchema: {
        type: "object",
        properties: {
          a: { type: "number" },
          b: { type: "number" }
        },
        required: ["a", "b"]
      }
    }, {
      name: "get_datasets",
      description: "Get all available Trieve datasets in your organization",
      inputSchema: {
        type: "object",
        properties: {},
        required: []
      }
    }, {
      name: "search_trieve",
      description: "Search within a Trieve dataset using a query",
      inputSchema: {
        type: "object",
        properties: {
          dataset_id: { type: "string" },
          query: { type: "string" }
        },
        required: ["dataset_id", "query"]
      }
    }]
  };
});

// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === "calculate_sum") {
    const { a, b } = request.params.arguments as { a: number; b: number };
    return { toolResult: a + b };
  }

  if (request.params.name === "get_datasets") {
    try {
      if (!process.env.TRIEVE_ORGANIZATION_ID) {
        throw new McpError(400, "TRIEVE_ORGANIZATION_ID is not set");
      }

      if (!process.env.TRIEVE_API_KEY) {
        throw new McpError(400, "TRIEVE_API_KEY is not set");
      }

      const response = await trieve.getDatasetsFromOrganization(
        process.env.TRIEVE_ORGANIZATION_ID
      );

      if (!response || !Array.isArray(response)) {
        throw new McpError(500, "Invalid response from Trieve API");
      }

      return {
        toolResult: {
          message: "Here are your available datasets:",
          datasets: response.map((dataset: DatasetResponse) => ({
            id: dataset.dataset.id,
            name: dataset.dataset.name,
            chunk_count: dataset.dataset_usage.chunk_count
          }))
        }
      };
    } catch (error) {
      console.error('Error fetching datasets:', error);
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(500, "Failed to fetch datasets: " + (error as Error).message);
    }
  }

  if (request.params.name === "search_trieve") {
    try {
      const { dataset_id, query } = request.params.arguments as { dataset_id: string; query: string };
      
      // Set the dataset ID for the SDK instance
      trieve.setDatasetId(dataset_id);

      // Make the search request
      const response = await trieve.search({
        query,
        search_type: 'hybrid',
        filters: {
          must: [{
            field: 'dataset_id',
            match_any: [dataset_id]
          }]
        }
      });

      if (!response || !response.chunks) {
        throw new McpError(500, "Invalid response format from Trieve API");
      }

      return {
        toolResult: {
          results: response.chunks.map((chunk: SearchChunk) => ({
            content: chunk.chunk.chunk_html || '',
            score: chunk.score || 0
          }))
        }
      };
    } catch (error) {
      console.error('Search error:', error);
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(500, `Search failed: ${(error as Error).message}`);
    }
  }

  throw new McpError(404, "Tool not found");
});

const transport = new StdioServerTransport();
await server.connect(transport);
