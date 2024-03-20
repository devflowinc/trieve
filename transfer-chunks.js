const res = await fetch("https://api.trieve.ai/api/chunk/search", {
  headers: {
    "content-type": "application/json",
    "tr-dataset": "982df67b-a95f-46f6-adfd-25ee7648c79c",
    Authorization: "tr-wPFc6qBAM9T6XEXDXLPFl2RhNZ8iUSZD",
  },
  body: '{"query":"test","page":1,"filters":{"must":[]},"search_type":"hybrid","get_collisions":true}',
  method: "POST",
});

const data = await res.json();

const score_chunks = data.score_chunks;

for (let i = 0; i < score_chunks.length; i++) {
  const chunk_data = score_chunks[i].metadata[0];

  const queued_chunk = await fetch("http://127.0.0.1:8090/api/chunk", {
    headers: {
      "content-type": "application/json",
      Authorization: "tr-jmTgOG67D3TlNwjmqXytxOgCT82n5Eay",
      "tr-dataset": "5495a77b-cd85-428e-960e-7f4c856885d4",
    },
    body: JSON.stringify({
      chunk_html: chunk_data.chunk_html,
      metadata: chunk_data.metadata,
    }),
    method: "POST",
  });

  if (queued_chunk.ok) {
    console.log("Chunk queued successfully");
  } else {
    console.error(
      "Chunk failed to queue",
      queued_chunk.status,
      await queued_chunk.text()
    );
  }
}
