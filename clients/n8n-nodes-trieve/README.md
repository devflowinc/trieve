# n8n-nodes-trieve

This contains the n8n nodes for using Trieve with n8n.

## Setup

Before using the Trieve node, you'll need to:

1. Create an account at [dashboard.trieve.ai](https://dashboard.trieve.ai)
2. Create a dataset in the dashboard
3. Generate an API key from your dashboard settings
4. Use this API and Dataset ID key when configuring the Trieve node credentials in n8n

## Chunk Operations

The Trieve node provides two main operations for working with chunks:

### Creating Chunks

To create a chunk in Trieve:

1. Add the Trieve node to your workflow
2. Select "Chunk" as the resource and "Create Chunk" as the operation
3. Fill in the required fields:
   - Chunk HTML: The content you want to store
   - Additional Fields:
     - Tag Set: Comma-separated list of tags to help organize your chunks (e.g., "important,project-x,documentation")
     - Tracking ID: A unique identifier for the chunk
     - Time Stamp: When the chunk was created
     - Metadata: Additional JSON data to store with the chunk

### Searching Chunks

To search through your chunks:

1. Add the Trieve node to your workflow
2. Select "Chunk" as the resource and "Search" as the operation
3. Configure your search:
   - Query: The text to search for
   - Search Type: Choose between Fulltext, Hybrid, or Semantic search
   - Page Size: Number of results to return
   - Filter: JSON filter to narrow down results

The search will return matching chunks along with their relevance scores and any matching highlights.

## Tool Call Operation

The tool_call operation allows you to define and execute custom functions with specific parameters. This is useful for creating structured decision-making workflows.

### Using Tool Call

To use the tool_call operation:

1. Add the Trieve node to your workflow
2. Select "Tool Call" as the resource and "Tool Call" as the operation
3. Configure the function:
   - Function Input: The input text to process
   - Function Name: A unique name for your function (e.g., "is_important", "classify_document")
   - Function Description: A clear description of what the function does
   - Parameters: Define one or more parameters for your function:
     - Name: The parameter name
     - Parameter Type: Choose between Boolean or Number
     - Description: Explain what the parameter represents

Example configuration (in json format):
```json
{
  "function_input": "This is an important document about project X",
  "function_name": "is_important",
  "function_description": "A function to determine if the input is important",
  "parameters": [
    {
      "name": "is_important",
      "parameter_type": "boolean",
      "description": "Whether the input is important"
    },
    {
      "name": "importance_score",
      "parameter_type": "number",
      "description": "A score from 0-10 indicating importance"
    }
  ]
}
```

The tool_call operation will return the parameters with their determined values based on the input text.

## Local development

You need the following installed on your development machine:

- Install n8n with:
  ```
  npm install n8n -g
  ```
- Run `n8n` to start n8n for the first time
- Loin to [http://localhost:5678](http://localhost:5678) and complete onboarding steps and this should create a folder in `~/.n8n.`
- To allow the node to be loaded locally run:

   ```
   mkdir ~/.n8n/custom/
   ln -s "$(pwd)/n8n-nodes-trieve ~/.n8n/custom/n8n-nodes-trieve
   ```

- Run `npm run build` inside the `n8n-nodes-trieve` directory
- Start `n8n` again, you should see the node in the sidebar.
- 
