import { useEffect, useState } from "react";
import { useTrieve } from "./TrieveProvider";

export const useChunkExtraContent = (productId: string | undefined) => {
  const trieve = useTrieve();
  const [extraContent, setExtraContent] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [aiLoading, setAILoading] = useState(false);

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
    setLoading(false);
  };

  const generateAIDescription = async () => {
    if (!productId) {
      console.info("Tried to generate AI description without id");
      return;
    }
    setLoading(true);
    setAILoading(true);
    const topic = await trieve.createTopic({
      owner_id: "shopify-enrich-content-block",
      first_user_message: "Describe this product",
      name: "Shopify Enrich Content Block",
    });

    const message = await trieve.createMessage({
      topic_id: topic.id,
      new_message_content:
        "Describe this product. Generate a description for an online shop. Keep it to 3 sentences maximum. Do not include an introduction or welcome message",
      use_group_search: true,
      filters: {
        must: [
          {
            field: "group_tracking_ids",
            match_all: [productId],
          },
        ],
      },
      llm_options: {
        stream_response: false,
      },
    });

    const response = message.split("||").at(1);
    if (!response) {
      console.error("No response from AI");
      return;
    }
    setExtraContent(response);
    setLoading(false);
    setAILoading(false);
  };

  const updateContent = async (content: string) => {
    if (!productId) {
      console.info("Tried to update product content without id");
      return;
    }
    setLoading(true);

    await trieve.createChunk({
      chunk_html: content,
      tracking_id: `${productId}-pdp-content`,
      upsert_by_tracking_id: true,
      group_tracking_ids: [productId],
    });
    setLoading(false);
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
    generateAIDescription,
    loading,
    aiLoading,
  };
};
