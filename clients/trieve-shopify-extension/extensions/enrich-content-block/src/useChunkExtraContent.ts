import { useEffect, useState } from "react";
import { useTrieve } from "./TrieveProvider";

export const useChunkExtraContent = (productId: string | undefined) => {
  const trieve = useTrieve();
  const [extraContent, setExtraContent] = useState<string | null>(null);

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
    if (!result.chunk_html) {
      return;
    }
    setExtraContent(result.chunk_html);
  };

  const updateContent = async (content: string) => {
    if (!productId) {
      console.info("Tried to update product content without id");
      return;
    }

    // const result = await trieve.updateChunkByTrackingId({
    //   trackingId: `${productId}-pdp-content`,
    //   chunk_html: content,
    // });
    //
    // if (!result.chunk_html) {
    //   return;
    // }
    // setExtraContent(result.chunk_html);
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
