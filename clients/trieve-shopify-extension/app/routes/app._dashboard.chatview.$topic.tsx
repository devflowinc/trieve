// app/routes/app.chat.$id.tsx
import { ChatIcon, PlusIcon } from "@shopify/polaris-icons";
import { LinksFunction, LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import {
  BlockStack,
  Card,
  InlineGrid,
  InlineStack,
  Page,
  Text,
} from "@shopify/polaris";
import { Suspense } from "react";
import { TrieveModalSearch } from "trieve-search-component";
import "trieve-search-component/styles";
import { useTrieve } from "app/context/trieveContext";
import styles from "../styles/chatview-styles.css?url";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import {
  MessageEventTimeline,
  SidebarEvent,
} from "app/components/chat/MessageEventTimeline";

export const links: LinksFunction = () => {
  return [{ rel: "stylesheet", href: styles }];
};

export async function loader({ params, request }: LoaderFunctionArgs) {
  const topicId = params.topic;
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  let messages = await trieve.getAllMessagesForTopic({
    messagesTopicId: topicId!,
  });

  return { topicId, messages };
}

export default function ChatRoute() {
  const { topicId, messages } = useLoaderData<typeof loader>();
  const { dataset, trieveKey } = useTrieve();

  const getEventLog = () => {
    const events: SidebarEvent[] = [];
    // create a "new chat" event
    if (messages.length > 0) {
      events.push({
        date: new Date(messages.at(0)!.created_at),
        type: "New Chat",
        icon: <PlusIcon width={20} fill="#000" height={20} />,
      });
    }

    messages.forEach((message) => {
      if (message.role === "user") {
        events.push({
          date: new Date(message.created_at),
          type: "User Message",
          icon: <ChatIcon width={20} fill="#000" height={20} />,
        });
      }
    });

    return events;
  };

  const events = getEventLog();

  return (
    <Page
      title="Chat Thread"
      fullWidth
      backAction={{
        content: "All Chats",
        url: "/app/chats",
      }}
    >
      <InlineGrid columns={{ md: 1, xl: "2fr 1fr" }} gap="400">
        <BlockStack align="space-between" gap="300">
          <InlineStack align="space-between" gap="300">
            <Suspense fallback={null}>
              {typeof window !== "undefined" && (
                <TrieveModalSearch
                  type="ecommerce"
                  defaultSearchMode="chat"
                  allowSwitchingModes={false}
                  apiKey={trieveKey.key}
                  previewTopicId={topicId}
                  inlineCarousel={true}
                  datasetId={dataset.id}
                  inline={true}
                  debounceMs={10}
                  analytics={false}
                  brandColor="#ae715e"
                  brandLogoImgSrcUrl="https://cdn.trieve.ai/component/flaviar/Uncle%20Flaviar.svg"
                  hidePrice={true}
                  hideChunkHtml={true}
                  useGroupSearch={true}
                />
              )}
            </Suspense>
          </InlineStack>
        </BlockStack>
        <BlockStack>
          <Card padding={"300"}>
            <BlockStack align="space-between" gap="300">
              <Text variant="headingLg" as="h2">
                Metadata
              </Text>
              <Text variant="bodyMd" as="h2">
                Message Length:{" "}
                <Text as="span">
                  {" "}
                  {
                    messages.filter((message) => message.role == "user").length
                  }{" "}
                </Text>
              </Text>
              <MessageEventTimeline events={events} />
            </BlockStack>
          </Card>
        </BlockStack>
      </InlineGrid>
    </Page>
  );
}
