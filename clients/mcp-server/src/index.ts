#!/usr/bin/env node
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ErrorCode,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  McpError,
  ReadResourceRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { TrieveSDK } from "trieve-ts-sdk";
import type { DatasetAndUsage, SearchResult } from "./types";
import dotenv from "dotenv";

// Parse command line arguments
function parseArgs() {
  const args = {
    apiKey: "",
    organizationId: "",
  };

  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--api-key" && i + 1 < process.argv.length) {
      args.apiKey = process.argv[++i];
    } else if (arg === "--org-id" && i + 1 < process.argv.length) {
      args.organizationId = process.argv[++i];
    }
  }

  return args;
}

// Load environment variables and CLI args
dotenv.config();
const args = parseArgs();

const TRIEVE_API_KEY = args.apiKey || process.env.TRIEVE_API_KEY;
const TRIEVE_ORGANIZATION_ID =
  args.organizationId || process.env.TRIEVE_ORGANIZATION_ID;

if (!TRIEVE_API_KEY || !TRIEVE_ORGANIZATION_ID) {
  console.error("Error: API key and organization ID are required.");
  console.error("Provide them either through environment variables:");
  console.error("  TRIEVE_API_KEY=<key> TRIEVE_ORGANIZATION_ID=<id>");
  console.error("Or through command line arguments:");
  console.error("  --api-key <key> --org-id <id>");
  process.exit(1);
}
// Initialize Trieve SDK client
const trieveClient = new TrieveSDK({
  apiKey: TRIEVE_API_KEY,
  organizationId: TRIEVE_ORGANIZATION_ID,
});

export class TrieveMcpServer {
  private server: Server;
  private trieveClient: TrieveSDK;
  private isConnected: boolean = false;

  constructor() {
    this.server = new Server(
      {
        name: "mcp-server-trieve",
        version: "0.1.0",
      },
      {
        capabilities: {
          resources: {
            enabled: true,
          },
          tools: {
            enabled: true,
          },
        },
      }
    );

    this.trieveClient = trieveClient;

    this.setupResourceHandlers();
    this.setupToolHandlers();

    // Enhanced error handling
    this.server.onerror = (error) => {
      console.error("[MCP Error]", error);
      if (!this.isConnected) {
        process.exit(1);
      }
    };

    // Cleanup handlers
    const cleanup = async () => {
      if (this.isConnected) {
        try {
          await this.server.close();
        } catch (error) {
          console.error("Error during shutdown:", error);
        }
      }
      process.exit(0);
    };

    process.on("SIGINT", cleanup);
    process.on("SIGTERM", cleanup);
    process.on("uncaughtException", (error) => {
      console.error("Uncaught exception:", error);
      cleanup();
    });
  }

  private setupResourceHandlers() {
    this.server.setRequestHandler(
      ListResourcesRequestSchema,
      async (request) => {
        return await this.handleListResources();
      }
    );

    this.server.setRequestHandler(
      ReadResourceRequestSchema,
      async (request) => {
        return await this.handleReadResource(request.params.uri);
      }
    );
  }

  private setupToolHandlers() {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return await this.handleListTools();
    });

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      return await this.handleCallTool(
        request.params.name,
        request.params.arguments || {}
      );
    });
  }

  async handleListResources() {
    try {
      if (!TRIEVE_ORGANIZATION_ID) {
        throw new McpError(
          ErrorCode.InternalError,
          "TRIEVE_ORGANIZATION_ID is required"
        );
      }

      const datasets = await this.trieveClient.getDatasetsFromOrganization(
        TRIEVE_ORGANIZATION_ID
      );

      return {
        resources: datasets.map(
          ({ dataset, dataset_usage }: DatasetAndUsage) => ({
            uri: `trieve://datasets/${dataset.id}`,
            name: dataset.name,
            description: `Dataset: ${dataset.name}\nChunks: ${dataset_usage.chunk_count}`,
            mimeType: "application/json",
            metadata: {
              id: dataset.id,
              name: dataset.name,
              chunk_count: dataset_usage.chunk_count,
              created_at: dataset.created_at,
              updated_at: dataset.updated_at,
            },
          })
        ),
      };
    } catch (error) {
      console.error("[List Resources Error]", error);
      throw new McpError(
        ErrorCode.InternalError,
        `Failed to fetch datasets: ${error}`
      );
    }
  }

  async handleReadResource(uri: string) {
    const match = uri.match(/^trieve:\/\/datasets\/([^/]+)$/);
    if (!match) {
      throw new McpError(
        ErrorCode.InvalidRequest,
        `Invalid URI format: ${uri}`
      );
    }

    const datasetId = match[1];

    try {
      const [dataset, usage] = await Promise.all([
        this.trieveClient.getDatasetById(datasetId),
        this.trieveClient.getDatasetUsageById(datasetId),
      ]);

      const datasetInfo = {
        dataset: {
          id: dataset.id,
          name: dataset.name,
          created_at: dataset.created_at,
          updated_at: dataset.updated_at,
          chunk_count: usage.chunk_count,
        },
      };

      return {
        contents: [
          {
            uri,
            mimeType: "application/json",
            text: JSON.stringify(datasetInfo, null, 2),
          },
        ],
      };
    } catch (error) {
      console.error("[Read Resource Error]", error);
      throw new McpError(
        ErrorCode.InternalError,
        `Failed to fetch dataset details: ${error}`
      );
    }
  }

  async handleListTools() {
    return {
      tools: [
        {
          name: "search",
          description:
            "Search through Trieve datasets using semantic, fulltext, hybrid or BM25 search with advanced filtering and highlighting options",
          inputSchema: {
            type: "object",
            properties: {
              query: {
                type: "string",
                description: "Search query to find relevant content",
              },
              datasetId: {
                type: "string",
                description: "ID of the dataset to search in",
              },
              searchType: {
                type: "string",
                enum: ["semantic", "fulltext", "hybrid", "bm25"],
                description:
                  "Search type: semantic (embeddings), fulltext (SPLADE), hybrid (both), or BM25 (keyword matching)",
                default: "semantic",
              },
              filters: {
                type: "object",
                description:
                  "Filter results using boolean logic on metadata fields and tags",
                properties: {
                  must: {
                    type: "array",
                    description: "All conditions must match (AND)",
                    items: {
                      type: "object",
                      properties: {
                        field: {
                          type: "string",
                          description:
                            "Field to filter on (e.g., tag_set, metadata.key)",
                        },
                        match_any: {
                          type: "array",
                          description: "Match any of these values (OR)",
                          items: {
                            type: "string",
                          },
                        },
                        match_all: {
                          type: "array",
                          description: "Match all of these values (AND)",
                          items: {
                            type: "string",
                          },
                        },
                        range: {
                          type: "object",
                          description: "Range filter for numeric fields",
                          properties: {
                            gt: { type: "number" },
                            gte: { type: "number" },
                            lt: { type: "number" },
                            lte: { type: "number" },
                          },
                        },
                      },
                    },
                  },
                  must_not: {
                    type: "array",
                    description: "None of these conditions can match (NOT)",
                    items: {
                      type: "object",
                      properties: {
                        field: {
                          type: "string",
                          description:
                            "Field to filter on (e.g., tag_set, metadata.key)",
                        },
                        match_any: {
                          type: "array",
                          description: "Match none of these values",
                          items: {
                            type: "string",
                          },
                        },
                      },
                    },
                  },
                  should: {
                    type: "array",
                    description: "At least one condition must match (OR)",
                    items: {
                      type: "object",
                      properties: {
                        field: {
                          type: "string",
                          description:
                            "Field to filter on (e.g., tag_set, metadata.key)",
                        },
                        match_any: {
                          type: "array",
                          description: "Match any of these values",
                          items: {
                            type: "string",
                          },
                        },
                      },
                    },
                  },
                },
              },
              highlightOptions: {
                type: "object",
                description: "Options for highlighting matching content",
                properties: {
                  highlightResults: {
                    type: "boolean",
                    description: "Add highlight markers to matching content",
                    default: true,
                  },
                  highlightThreshold: {
                    type: "number",
                    description:
                      "Score threshold for including highlights (0.0-1.0)",
                    default: 0.8,
                  },
                  highlightMaxLength: {
                    type: "integer",
                    description: "Maximum tokens in a single highlight",
                    default: 8,
                  },
                  highlightMaxNum: {
                    type: "integer",
                    description: "Maximum number of highlights per chunk",
                    default: 3,
                  },
                  preTag: {
                    type: "string",
                    description: "HTML tag before highlights",
                    default: "<mark><b>",
                  },
                  postTag: {
                    type: "string",
                    description: "HTML tag after highlights",
                    default: "</b></mark>",
                  },
                },
              },
              page: {
                type: "integer",
                description: "Page number (1-indexed)",
                default: 1,
              },
              pageSize: {
                type: "integer",
                description: "Results per page",
                default: 10,
              },
              scoreThreshold: {
                type: "number",
                description: "Minimum score threshold for including results",
              },
              useWeights: {
                type: "boolean",
                description: "Use chunk weights in ranking",
                default: true,
              },
              getTotalPages: {
                type: "boolean",
                description: "Get total page count (adds latency)",
                default: false,
              },
              slimChunks: {
                type: "boolean",
                description:
                  "Exclude content/chunk_html from results for faster response",
                default: false,
              },
            },
            required: ["query", "datasetId"],
          },
        },
      ],
    };
  }

  async handleCallTool(name: string, args: any) {
    try {
      if (name !== "search") {
        throw new McpError(ErrorCode.MethodNotFound, `Unknown tool: ${name}`);
      }

      if (!args || typeof args !== "object") {
        throw new McpError(
          ErrorCode.InvalidParams,
          "Arguments must be an object"
        );
      }

      const { query, datasetId } = args;

      if (typeof query !== "string" || !query.trim()) {
        throw new McpError(
          ErrorCode.InvalidParams,
          "query must be a non-empty string"
        );
      }

      if (typeof datasetId !== "string" || !datasetId.trim()) {
        throw new McpError(
          ErrorCode.InvalidParams,
          "datasetId must be a non-empty string"
        );
      }

      try {
        // Set the dataset ID for this search
        this.trieveClient.datasetId = datasetId;

        // Convert parameters to Trieve API format
        const searchParams: any = {
          query: args.query,
          search_type: args.searchType || "semantic",
          page: args.page || 1,
          page_size: args.pageSize || 10,
          filters: args.filters,
          get_total_pages: args.getTotalPages,
          slim_chunks: args.slimChunks,
          score_threshold: args.scoreThreshold,
          use_weights: args.useWeights,
        };

        // Add highlight options if provided
        if (args.highlightOptions) {
          searchParams.highlight_results =
            args.highlightOptions.highlightResults;
          searchParams.highlight_threshold =
            args.highlightOptions.highlightThreshold;
          searchParams.highlight_max_length =
            searchParams.highlight_max_length =
              args.highlightOptions.highlightMaxLength;
          searchParams.highlight_max_num =
            args.highlightOptions.highlightMaxNum;
          searchParams.pre_tag = args.highlightOptions.preTag;
          searchParams.post_tag = args.highlightOptions.postTag;
        }

        const results: SearchResult =
          await this.trieveClient.search(searchParams);

        // Transform results to a more concise format
        const transformedResults = {
          id: results.id,
          chunks: results.chunks.map((chunk) => {
            // Safely access metadata with type assertions
            const chunkData = chunk.chunk as any;
            const metadata = (chunkData.metadata || {}) as Record<string, any>;

            return {
              score: chunk.score,
              highlights: chunk.highlights,
              metadata: {
                title: metadata.heading || metadata.title || "",
                url: metadata.url || chunkData.link || "",
                description: metadata.description || "",
              },
              content: chunkData.content || chunkData.chunk_html || "",
              tags: Array.isArray(chunkData.tag_set) ? chunkData.tag_set : [],
            };
          }),
        };

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(transformedResults, null, 2),
            },
          ],
        };
      } catch (error) {
        console.error("[Search Error]", error);
        throw new McpError(
          ErrorCode.InternalError,
          `Search failed: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
      }
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        error instanceof Error ? error.message : String(error)
      );
    }
  }

  async run() {
    try {
      // Start MCP server with timeout
      const transport = new StdioServerTransport();
      await this.server.connect(transport);
      this.isConnected = true;

      // Send startup status
      await this.server.notification({
        method: "server/status",
        params: {
          type: "startup",
          status: "running",
          transport: "stdio",
        },
      });

      // Keep process alive and handle termination
      process.stdin.resume();
    } catch (error) {
      console.error("[Startup Error]", error);
      throw {
        code: -32000,
        message: "Server startup failed",
        data: {
          error: error instanceof Error ? error.message : String(error),
        },
      };
    }
  }
}

// Only start the server if this file is run directly or via npx
if (
  import.meta.url === `file://${process.argv[1]}` ||
  process.argv[1]?.includes("mcp-server-trieve") ||
  process.argv[1]?.includes("trieve-mcp-server")
) {
  const server = new TrieveMcpServer();
  server.run().catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
  });
}
