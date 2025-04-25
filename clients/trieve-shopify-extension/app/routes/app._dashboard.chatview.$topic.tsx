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
import { parseCustomDateString } from "app/utils/formatting";

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
  const { trieve, dataset, trieveKey } = useTrieve();

  const getEventLog = () => {
    const events: SidebarEvent[] = [];
    // create a "new chat" event
    const sortedMessages = messages.sort(
      (a, b) =>
        new Date(b.created_at).getUTCSeconds() -
        new Date(a.created_at).getUTCSeconds(),
    );

    sortedMessages.forEach((message) => {
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
          date: new Date(parseCustomDateString(event.updated_at, false)),
          type: "Click on Product",
          icon: <CursorIcon width={20} fill="#000" height={20} />,
        });
      }
      if (event.event_name === "site-add_to_cart") {
        events.push({
          date: new Date(parseCustomDateString(event.updated_at, false)),
          type: "Add To Cart",
          highlight: true,
          icon: <CartIcon width={20} fill="#000" height={20} />,
        });
      }
    });

    const result = events.sort((a, b) => a.date.getTime() - b.date.getTime());

    if (sortedMessages.length > 0) {
      result.unshift({
        date: new Date(sortedMessages.at(0)!.created_at),
        type: "New Chat",
        additional: format(
          new Date(
            parseCustomDateString(sortedMessages.at(0)!.created_at, true),
          ),
          "h:m aa, LLL d y",
        ),
        icon: <PlusIcon width={20} fill="#000" height={20} />,
      });
    }
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
                  baseUrl={trieve.trieve.baseUrl}
                  cssRelease="beta"
                  useGroupSearch={true}
                  hideOpenButton={true}
                  defaultSearchMode="chat"
                  apiKey={trieveKey.key}
                  datasetId={dataset.id}
                  // skip zIndex
                  // skip defaultAiQuestions
                  brandColor="#ae715e" // TODO: get this from the settings
                  brandLogoImgSrcUrl="https://cdn.trieve.ai/trieve-logo.png" // TODO: brandLogoImgSrcUrl from settings
                  // skip chatPlaceholder
                  // skip suggestedQueries
                  // skip followQuestions
                  // skip numberOfSuggestions
                  openLinksInNewTab={true}
                  // TODO: get theme from settings
                  showTrieve={false}
                  // skip geCartQuantity
                  // skip searchOptions
                  // skip chatFilters
                  // defaultCurrency TODO: get this from settings
                  // ---
                  // BELOW ARE UNIQUE TO THE ANALYTICS CHAT VIEW
                  allowSwitchingModes={false}
                  previewTopicId={topicId}
                  inlineCarousel={true}
                  inline={true}
                  debounceMs={10}
                  analytics={false}
                  hidePrice={true}
                  hideChunkHtml={true}
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
