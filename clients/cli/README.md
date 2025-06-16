<p align="center">
  <img height="100" src="https://cdn.trieve.ai/trieve-logo.png" alt="Trieve Logo">
</p>
<p align="center">
<strong><a href="https://dashboard.trieve.ai">Sign Up (1k chunks free)</a> | <a href="https://docs.trieve.ai">Documentation</a> | <a href="https://discord.gg/eBJXXZDB8z">Discord</a> | <a href="https://matrix.to/#/#trieve-general:trieve.ai">Matrix</a>
</strong>
</p>

<p align="center">
    <a href="https://github.com/devflowinc/trieve/stargazers">
        <img src="https://img.shields.io/github/stars/devflowinc/trieve.svg?style=flat&color=yellow" alt="Github stars"/>
    </a>
    <a href="https://discord.gg/CuJVfgZf54">
        <img src="https://img.shields.io/discord/1130153053056684123.svg?label=Discord&logo=Discord&colorB=7289da&style=flat" alt="Join Discord"/>
    </a>
    <a href="https://matrix.to/#/#trieve-general:trieve.ai">
        <img src="https://img.shields.io/badge/matrix-join-purple?style=flat&logo=matrix&logocolor=white" alt="Join Matrix"/>
    </a>
</p>

# Trieve CLI

A command-line interface for interacting with the Trieve API. The CLI enables users to upload files, check upload status, ask questions against their knowledge base, and configure their Trieve setup.

## Installation

```bash
npm install -g trieve-cli
```

## Configuration

Before using the CLI, you need to configure it with your Trieve credentials:

```bash
trieve configure
```

This interactive command will prompt you for:

- Your Trieve Organization ID
- Your Trieve Dataset ID
- Your Trieve API Key
- User ID (for topic ownership)

Alternatively, you can set these as environment variables:

- `TRIEVE_ORGANIZATION_ID`
- `TRIEVE_DATASET_ID`
- `TRIEVE_API_KEY`

## Commands

### Upload Files

Upload a file to your Trieve dataset:

```bash
trieve upload [filePath] [-t, --tracking-id <trackingId>]
```

If no file path is provided, the CLI will prompt you to enter one interactively.

### Check Upload Status

Check the status of your uploaded files:

```bash
trieve check-upload-status [-t, --tracking-id <trackingId>]
```

Without options, this will display an interactive menu to select files to check. If a tracking ID is provided, it will check the status of that specific file.

### Ask Questions

Ask a question against your Trieve dataset:

```bash
trieve ask [question]
```

If no question is provided, the CLI will prompt you to enter one interactively. The response will be streamed back with reference information that you can expand by pressing 'j'.

### Update Tool Configuration

Customize the RAG system prompt and tool configurations:

```bash
trieve update-tool-config [-t, --tool-description <toolDescription>] [-q, --query-description <queryDescription>] [-s, --system-prompt <systemPrompt>]
```

This allows you to customize:

- Tool description: Instructions for when the LLM should use the search tool
- Query description: How the LLM should formulate search queries
- System prompt: Custom system prompt for the AI assistant

## Features

- **🔒 Secure Configuration**: Local storage of API keys and configuration
- **📤 File Uploads**: Upload documents to your Trieve dataset
- **📋 Status Tracking**: Monitor the processing status of uploaded files
- **🤔 Interactive Q&A**: Ask questions and receive answers based on your uploaded documents
- **📚 Reference Display**: View source references for answers with expandable details
- **🔧 Customizable RAG**: Configure system prompts and tool behavior

## Examples

### Upload a PDF and ask questions about it:

```bash
# Upload a document
trieve upload ./documents/report.pdf

# Check if processing is complete
trieve check-upload-status

# Ask a question about the content
trieve ask "What are the key findings in the report?"
```

### Customize the RAG behavior:

```bash
# Update the tool configuration for more specific search behavior
trieve update-tool-config --query-description "Create precise search queries focusing on technical terms and definitions"
```

## Development

To build the CLI from source:

```bash
# Install dependencies
npm install

# Build the TypeScript code
npm run build

# Run locally
npm start
```

## License

MIT
