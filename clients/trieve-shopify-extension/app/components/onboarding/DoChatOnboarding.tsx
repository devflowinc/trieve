import { Button, Text } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { allChatsQuery } from "app/queries/analytics/chat";
import { OnboardingBody } from "app/utils/onboarding";
import { useEffect } from "react";

export const DoChatOnboarding: OnboardingBody = ({
  broadcastCompletion,
  goToNextStep,
}) => {
  const { trieve } = useTrieve();

  const { data: chats } = useQuery(allChatsQuery(trieve, {}, 1));

  useEffect(() => {
    if (chats && chats.topics.length > 0) {
      if (broadcastCompletion) {
        broadcastCompletion();
      }
    }
  });

  const complete = chats?.topics.length && chats?.topics.length > 0;

  return (
    <div className="grid w-full py-2">
      <div className="flex flex-col gap-1">
        <Text as="p">
          {complete === true
            ? "Component Added"
            : "Chat with your products using the Trieve chat widget."}
        </Text>
      </div>
    </div>
  );
};
