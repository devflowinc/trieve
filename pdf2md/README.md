<p align="center">
  <img height="100" src="https://trieve.b-cdn.net/trieve-logo.png" alt="Trieve Logo">
</p>
<p align="center">
<strong><a href="https://pdf2md.trieve.ai/redoc">API reference</a> | <a href="https://cal.com/nick.k/meet">Meet a Maintainer</a> | <a href="https://discord.gg/eBJXXZDB8z">Discord</a> | <a href="https://matrix.to/#/#trieve-general:trieve.ai">Matrix</a> | <a href="mailto:humans@trieve.ai">humans@trieve.ai</a>
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

<h1 align="center">🦀 PDF2MD 🦀</h1>

<h2 align="center">
    <b>Self-hostable API server and pipeline for converting PDF's to markdown using thrifty large language vision models like GPT-4o-mini and gemini-flash-1.5.</b>
</h2>

<h4 align="center">Written in Rust. Try at <a href="https://pdf2md.trieve.ai">pdf2md.trieve.ai</a>.</h4>

[![PDF2MD service preview](https://cdn.trieve.ai/pdf2md/pdf2md-preview.webp)](https://pdf2md.trieve.ai)

## The Stack

There's no compelling reason why Rust is necessary for this, but we wanted to have some fun 😜. Everything is free and open source. You can self-host easily with `docker-compose` or `kube` following the [SELF-HOSTING guide here](https://github.com/devflowinc/trieve/tree/main/pdf2md/SELF-HOSTING.md).

- [minijinja templates](https://github.com/mitsuhiko/minijinja) for the [UI](https://pdf2md.trieve.ai)
    - there was no way I was going to write more JSX
- [PDFObject](https://github.com/pipwerks/pdfobject) to view PDF's in the [demo UI](https://pdf2md.trieve.ai).
- [actix/actix-web](https://github.com/actix/actix-web) for the HTTP server
- [fun redis queue macro system](https://github.com/devflowinc/trieve/blob/main/pdf2md/server/src/operators/redis.rs#L7-L62) for worker pattern async processing
    - redis queues are a core part of our infra for Trieve, but we made our system a lot more repeatable with this macro
    - there will be a future release of this macro in an isolated crate
- [Clickhouse](https://github.com/ClickHouse/ClickHouse) for task storage
    - we have had a surprising amount of Postgres issues (especially write locks) building Trieve, so Clickhouse as the primary data store here is cool
- [MinIO S3](https://github.com/minio/minio) for file storage

## How does PDF2MD work?

Workers horizontally scale on-demand to handle high volume periods. Usually `chunk-worker` needs to scale before `supervisor-worker`. Pages for a given `Task` stream in as the `chunk-worker` calls out to the LLM to get markdown for them.

### 1. HTTP server

1. HTTP server receives a base64 encoded PDF and decodes it
3. Creates `FileTask` for document in ClickHouse
4. Adds `FileTask` along with the base64 encoded file to `files_to_process` queue in Redis

### 2. Supervisor Worker

1. `supervisor-worker` continuously polls the `files_to_process` Redis queue until it grabs a `FileTask` and its base64
2. Decodes the base64 into a PDF and puts the PDf into S3
3. Splits the PDF into pages, converts them to JPEGs
4. Puts each JPEG page image into S3
5. Pushes a `ChunkingTask` for each page into the `files_to_chunk` Redis queue

### 3. Chunk Worker

1. `chunk-worker` continuously polls the `files_to_chunk` Redis queue until it grabs a `ChunkingTask`
2. Gets its page image from S3
3. Sends the image to the LLM provider at `LLM_BASE_URL` along with the `prompt` and `model` on the request to get markdown
4. Updates the task with the markdown for the page

## Why Make This?

Trieve has used [apache tika](https://tika.apache.org/) to process various filetypes for the past year which means that files with complex layouts and diagrams have been poorly ingested. 

We saw [OmniAI](https://github.com/getomni-ai) launch [xerox](https://github.com/getomni-ai/zerox) and show that 4o-mini was a viable and cheap way to handle these filetypes and decided it was time to integrate something better than Tika into Trieve.

We previously lightly contributed to [Chunkr](https://github.com/lumina-ai-inc/chunkr) which is a more advanced system that leverages layout detection and dedicated OCR models to process documents, but still felt the need to build something ourselves since it was a bit complex to work into Trieve's local dev and self-hosting setup. Xerox's approach using just a VLLM was ideal and the path we went with.

We wrote our own API server and pipeline using Rust, Redis queues, and Clickhouse in the Trieve-style to achieve this. Try it using our demo UI hosted at [pdf2md.trieve.ai](https://pdf2md.trieve.ai).

## Roadmap

Please contribute if you can! We could use help 🙏.

1. Rename everything from `chunk` to `page` because we eventually decided that we would only deal PDF --> Markdown conversion and not chunking. Consider using [chonkie](https://github.com/bhavnicksm/chonkie) with the markdown output for this.
2. Use [Clickhouse MergeTree](https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree) instead of updating `Task`'s in Clickhouse as that's more correct.
3. `supervisor-worker` can get overwhelmed when it receives a large PDF as splitting into pages can take a while. There should be something better here.
4. Users should be able to send a URL to a file instead of base64 encoding it if they have one because that's easier. 
5. Users should be able to point `PDF2MD` at an S3 bucket and let it process all of them automatically instead of having to send each file 1 by 1 🤮.

---

Made with ❤️ in San Francisco
