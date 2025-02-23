import { useEffect, useState } from "react";
import { RecommendationsConfig } from "./Recommendations";
import {
  ChunkMetadata,
  GetRecommendedChunksData,
  RecommendChunksResponseBody,
  ScoreChunk,
  SlimChunkMetadataWithArrayTagSet,
} from "trieve-ts-sdk";

export const useRecommendations = (config: RecommendationsConfig) => {
  const [status, setStatus] = useState<"loading" | "error" | "success">(
    "loading",
  );

  const [results, setResults] = useState<ChunkMetadata[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setStatus("loading");

    const getData = async () => {
      try {
        const response = await fetch(config.baseUrl + "/api/chunk/recommend", {
          method: "POST",
          body: JSON.stringify({
            positive_tracking_ids: [config.productId],
          } satisfies GetRecommendedChunksData["requestBody"]),
          headers: {
            "TR-Dataset": config.datasetId,
            Authorization: config.apiKey,
            "Content-Type": "application/json",
          },
        });

        if (!response.ok) {
          setStatus("error");

          try {
            const error = (await response.json()) as {
              message: string;
            };
            setError(error.message);
          } catch {
            setError("Something went wrong");
          }

          return;
        }

        const data = (await response.json()) as {
          chunks: { chunk: ChunkMetadata }[];
        };
        console.log(data);
        setResults(data.chunks.map((c) => c.chunk));
        setStatus("success");
      } catch {
        setStatus("error");
        setError("Something went wrong");
      }
    };

    getData();
  }, [config.productId]);

  return { status, results, error };
};
