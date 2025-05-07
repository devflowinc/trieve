import { useEffect, useState } from "react";
import { useTrieve } from "./TrieveProvider";

export const useChunkExtraContent = (productId: string | undefined) => {
  const trieve = useTrieve();
  const [extraContent, setExtraContent] = useState<string>("");

  // Fetch current product extra content chunk
  const getData = async () => {
    if (!productId) {
      console.info("Tried to fetch product content without id");
      return;
    }
    try {
      const result = await trieve.getChunkByTrackingId({
        trackingId: `${productId}-pdp-content`,
      });

      if (!result) {
        setExtraContent("");
        return;
      }
      if (!result.chunk_html) {
        setExtraContent("");
        return;
      }
      setExtraContent(result.chunk_html);
    } catch {
      setExtraContent("");
    }
  };

  const updateContent = async (content: string) => {
    if (!productId) {
      console.info("Tried to update product content without id");
      return;
    }

    const result = await trieve.createChunk({
      chunk_html: content,
      tracking_id: `${productId}-pdp-content`,
      upsert_by_tracking_id: true,
      weight: 100,
      group_tracking_ids: [productId],
    });
  };

  useEffect(() => {
    if (!productId) {
      return;
    }
    getData();
  }, [productId]);

  return {
    extraContent,
    updateContent,
  };
};
