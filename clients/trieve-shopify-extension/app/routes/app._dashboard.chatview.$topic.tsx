import {
  CartIcon,
  ChatIcon,
  CursorIcon,
  PlusIcon,
} from "@shopify/polaris-icons";
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
import { EventsForTopicResponse } from "trieve-ts-sdk";
import { format } from "date-fns";

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

  let topicEvents = (await trieve.getRagAnalytics({
    type: "events_for_topic",
    topic_id: topicId!,
  })) as EventsForTopicResponse;

  return { topicId, messages, topicEvents: topicEvents.events };
}

export default function ChatRoute() {
  const { topicId, messages, topicEvents } = useLoaderData<typeof loader>();
  const { dataset, trieveKey } = useTrieve();

  const getEventLog = () => {
    const events: SidebarEvent[] = [];
    // create a "new chat" event
    const sortedMessages = messages.sort(
      (a, b) =>
        new Date(b.created_at).getUTCSeconds() -
        new Date(a.created_at).getUTCSeconds(),
    );
    if (sortedMessages.length > 0) {
      events.push({
        date: new Date(sortedMessages.at(0)!.created_at),
        type: "New Chat",
        additional: format(
          new Date(sortedMessages.at(0)!.created_at),
          "h:m aa, LLL d y",
        ),
        icon: <PlusIcon width={20} fill="#000" height={20} />,
      });
    }

    messages.forEach((message) => {
      if (message.role === "user") {
        events.push({
          date: new Date(message.created_at),
          type: "User Message",
          additional: message.content,
          icon: <ChatIcon width={20} fill="#000" height={20} />,
        });
      }
    });

    topicEvents.forEach((event) => {
      if (event.event_name === "View") return;
      if (event.event_name === "Click") {
        events.push({
          date: new Date(event.created_at),
          type: "Click on Product",
          icon: <CursorIcon width={20} fill="#000" height={20} />,
        });
      }
      if (event.event_name === "site-add_to_cart") {
        events.push({
          date: new Date(event.created_at),
          type: "Add To Cart",
          icon: <CartIcon width={20} fill="#000" height={20} />,
        });
      }
    });

    const result = events.sort(
      (a, b) => b.date.getUTCSeconds() - a.date.getUTCSeconds(),
    );
    return result;
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
