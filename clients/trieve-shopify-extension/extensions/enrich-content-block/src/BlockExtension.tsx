import React, { useEffect, useState } from "react";
import {
  reactExtension,
  BlockStack,
  AdminBlock,
  useApi,
  Text,
  Button,
  InlineStack,
  Banner,
  TextArea,
  Icon,
  Box,
} from "@shopify/ui-extensions-react/admin";
import { TrieveProvider, useTrieve } from "./TrieveProvider";
import { ChunkMetadata } from "trieve-ts-sdk";

const TARGET = "admin.product-details.block.render";

function extractShopifyProductId(gid: string): string | undefined {
  const parts = gid.substring(6).split("/"); // Remove "gid://" and then split
  if (parts.length === 3 && parts[0] === "shopify" && parts[1] === "Product") {
    return parts[2];
  }
  return undefined;
}

export default reactExtension(TARGET, () => (
  <TrieveProvider>
    <App />
  </TrieveProvider>
));

function App() {
  const { data } = useApi(TARGET);
  const productId = extractShopifyProductId(data.selected[0].id);
  const [content, setContent] = useState<ChunkMetadata[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [showSuccess, setShowSuccess] = useState(false);
  const trieve = useTrieve();
  const [extraContent, setExtraContent] = useState<ChunkMetadata[]>([]);
  const [aiLoading, setAILoading] = useState(false);

  // Fetch current product extra content chunk
  const getData = async () => {
    if (!productId) {
      console.info("Tried to fetch product content without id");
      return;
    }
    try {
      const result = await trieve.scroll({
        filters: {
          should: [
            {
              field: "tag_set",
              match_all: [`${productId}-pdp-content`],
            },
            {
              tracking_ids: [`${productId}-pdp-content`],
            },
          ],
        },
        page_size: 20,
      });
      if (!result) {
        setExtraContent([]);
        return;
      }
      setExtraContent(result.chunks);
    } catch {
      setExtraContent([]);
    }
  };

  const generateAIDescription = async () => {
    if (!productId) {
      console.info("Tried to generate AI description without id");
      return;
    }
    setAILoading(true);
    const topic = await trieve.createTopic({
      owner_id: "shopify-enrich-content-block",
      first_user_message: "Describe this product",
      name: "Shopify Enrich Content Block",
    });

    const message = await trieve.createMessage({
      topic_id: topic.id,
      new_message_content:
        "Describe this product to add extra context to an LLM. Generate a description for an online shop. Keep it to 3 sentences maximum. Do not include an introduction or welcome message",
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

    const chunk = await trieve.createChunk({
      chunk_html: response,
      tag_set: [`${productId}-pdp-content`],
      group_tracking_ids: [productId],
    });
    // Check if chunk.chunk_metadata is a list
    if (!Array.isArray(chunk.chunk_metadata)) {
      setExtraContent((prev) => {
        return [chunk.chunk_metadata, ...prev];
      });
      setShowSuccess(true);
    }
    setAILoading(false);
  };

  useEffect(() => {
    if (!productId) {
      return;
    }
    getData();
  }, [productId]);

  const [indexBeingEdited, setIndexBeingEdited] = useState<number | null>(null);

  useEffect(() => {
    if (extraContent) {
      setContent(extraContent);
    }
  }, [extraContent]);

  const upsertContent = (chunk: ChunkMetadata) => {
    setIndexBeingEdited(null);
    if (chunk.id != "") {
      trieve.updateChunk({
        chunk_id: chunk.id,
        chunk_html: chunk.chunk_html,
      });
    } else if (productId) {
      trieve.createChunk({
        chunk_html: chunk.chunk_html,
        tag_set: [`${productId}-pdp-content`],
        group_tracking_ids: [productId],
      });
    }
  };

  return (
    <AdminBlock title="AI Context">
      <BlockStack gap="base">
        {showSuccess && (
          <Banner tone="success" onDismiss={() => setShowSuccess(false)}>
            Content saved successfully
          </Banner>
        )}
        <InlineStack inlineAlignment="space-between" blockAlignment="center">
          <Box inlineSize="80%">
            <Text>Product context for the AI</Text>
          </Box>
          <InlineStack
            blockAlignment="center"
            inlineAlignment="end"
            gap="base base"
          >
            <Button disabled={aiLoading} onPress={generateAIDescription}>
              <InlineStack blockAlignment="center">
                <Icon name="WandMinor" />
                {aiLoading ? "Generating..." : "Generate AI Context"}
              </InlineStack>
            </Button>

            <Button
              onPress={() => {
                setExtraContent((prev) => [
                  {
                    id: "",
                    chunk_html: "",
                    tag_set: [`${productId}-pdp-content`],
                    group_tracking_ids: [productId],
                    created_at: "",
                    updated_at: "",
                    dataset_id: "",
                    weight: 1, // Just to make the lsp stop
                  },
                  ...prev,
                ]);
                setIndexBeingEdited(0);
                setCurrentPage(1);
              }}
            >
              <InlineStack blockAlignment="center">
                <Icon name="PlusMinor" />
                Add Context
              </InlineStack>
            </Button>
          </InlineStack>
        </InlineStack>
        <Box>
          {content.map((chunk, index) => {
            if (index != currentPage - 1) {
              return null;
            }

            return (
              <Box key={index} padding="base small">
                <InlineStack
                  blockAlignment="center"
                  inlineAlignment="space-between"
                  inlineSize="100%"
                  gap="large"
                >
                  <Box
                    inlineSize={`${index === indexBeingEdited ? "100%" : "75%"}`}
                  >
                    {index === indexBeingEdited ? (
                      <TextArea
                        rows={4}
                        label=""
                        value={chunk.chunk_html ?? ""}
                        onChange={(value) => {
                          // updateContent(index, value);
                          setContent((prevContent) =>
                            prevContent.map((prevChunk) =>
                              prevChunk.id == chunk.id
                                ? { ...prevChunk, chunk_html: value }
                                : prevChunk,
                            ),
                          );
                        }}
                      />
                    ) : (
                      <Text>{chunk.chunk_html}</Text>
                    )}
                  </Box>
                  <Box inlineSize="25%">
                    <InlineStack
                      inlineSize="100%"
                      inlineAlignment="end"
                      blockAlignment="center"
                    >
                      {index === indexBeingEdited ? (
                        <>
                          <Button
                            onClick={() => {
                              upsertContent(chunk);
                            }}
                            variant="primary"
                          >
                            <Text>Finish</Text>
                          </Button>
                        </>
                      ) : (
                        <>
                          <Button
                            onClick={() => {
                              setIndexBeingEdited(index);
                            }}
                            variant="tertiary"
                          >
                            <Icon name="EditMinor" />
                          </Button>
                          <Button
                            onClick={() => {
                              trieve.deleteChunkById({
                                chunkId: chunk.id,
                              });
                              setContent((prevContent) =>
                                prevContent.filter(
                                  (prevChunk) => prevChunk.id != chunk.id,
                                ),
                              );
                              if (index === content.length - 1) {
                                setCurrentPage((prev) => prev - 1);
                              }
                            }}
                            variant="tertiary"
                          >
                            <Icon name="DeleteMinor" />
                          </Button>
                        </>
                      )}
                    </InlineStack>
                  </Box>
                </InlineStack>
              </Box>
            );
          })}
        </Box>
        <InlineStack
          paddingBlockStart="large"
          blockAlignment="center"
          inlineAlignment="center"
        >
          <Button
            onPress={() => setCurrentPage((prev) => prev - 1)}
            disabled={currentPage === 1}
          >
            <Icon name="ChevronLeftMinor" />
          </Button>
          <InlineStack
            inlineSize={50}
            blockAlignment="center"
            inlineAlignment="center"
          >
            <Text>
              {currentPage} / {content.length}
            </Text>
          </InlineStack>
          <Button
            onPress={() => setCurrentPage((prev) => prev + 1)}
            disabled={currentPage >= content.length}
          >
            <Icon name="ChevronRightMinor" />
          </Button>
        </InlineStack>
      </BlockStack>
    </AdminBlock>
  );
}
