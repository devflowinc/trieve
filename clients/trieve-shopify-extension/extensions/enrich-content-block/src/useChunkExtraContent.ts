import { useEffect, useState } from "react";
import { useTrieve } from "./TrieveProvider";

export const useChunkExtraContent = (productId: string | undefined) => {
  const trieve = useTrieve();
  const [extraContent, setExtraContent] = useState<string>("");

  const [loading, setLoading] = useState(true);

  // Fetch current product extra content chunk
  const getData = async () => {
    if (!productId) {
      console.info("Tried to fetch product content without id");
      return;
    }
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
  };

  const updateContent = async (content: string) => {
    if (!productId) {
      console.info("Tried to update product content without id");
      return;
    }

    const result = await trieve.createChunk({
      chunk_html: content,
      tracking_id: productId,
      upsert_by_tracking_id: true,
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
    loading,
    updateContent,
  };
};
