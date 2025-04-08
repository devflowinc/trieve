import { Button, Text } from "@shopify/polaris";
import { CheckIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { allChatsQuery } from "app/queries/analytics/chat";
import { OnboardingBody } from "app/utils/onboarding";
import { useShopName } from "app/utils/useShopName";
import { trackCustomerEvent } from "app/processors/shopifyTrackers";
import { useEffect } from "react";

export const DoChatOnboarding: OnboardingBody = ({ broadcastCompletion }) => {
  const { trieve } = useTrieve();

  const { data: chats } = useQuery(allChatsQuery(trieve, {}, 1));
  const shopname = useShopName();
  const shopUrl = shopname ? `https://${shopname}` : null;

  useEffect(() => {
    if (chats && chats.topics.length > 0) {
      if (broadcastCompletion) {
        if (trieve.organizationId && trieve.trieve.apiKey != null) {
          trackCustomerEvent(
            trieve.trieve.baseUrl,
            {
              organization_id: trieve.organizationId,
              store_name: "",
              event_type: "fist_chat_completed",
            },
            trieve.organizationId,
            trieve.trieve.apiKey,
          );
        }
        broadcastCompletion();
      }
    }
  }, [chats]);

  const complete =
    chats?.topics.length !== undefined && chats?.topics.length > 0;

  const viewStore = () => {
    if (shopUrl) {
      window.open(shopUrl, "_blank");
    }
  };

  return (
    <div className="grid w-full place-items-center h-[180px] pb-4 px-4 py-2">
      <div className="flex flex-col gap-2 items-center">
        {complete === false && (
          <Text as="p">
            Chat with your products using the Trieve chat widget.
          </Text>
        )}
        {complete && (
          <div className="w-full flex flex-col justify-center items-center">
            <CheckIcon
              fill="#2A845A"
              color="#2A845A"
              style={{ height: "50px" }}
            />
            <Text as="p">
              {chats.topics.length}{" "}
              {chats.topics.length === 1 ? "conversation" : "conversations"}{" "}
              completed
            </Text>
          </div>
        )}

        {shopUrl && !complete && (
          <Button onClick={viewStore}>View Store</Button>
        )}
      </div>
    </div>
  );
};
