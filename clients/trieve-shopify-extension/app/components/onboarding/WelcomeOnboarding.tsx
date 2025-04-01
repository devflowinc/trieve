import { Spinner, Text } from "@shopify/polaris";
import { CheckIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { crawlStatusOnboardQuery } from "app/queries/onboarding";
import { OnboardingBody } from "app/utils/onboarding";
import { useEffect } from "react";

export const WelcomeOnboarding: OnboardingBody = ({ broadcastCompletion }) => {
  const adminApi = useClientAdminApi();

  const { data } = useQuery({
    ...crawlStatusOnboardQuery(adminApi),
    refetchInterval: 1000,
    placeholderData: {
      chunkCount: 0,
      done: false,
    },
  });

  useEffect(() => {
    if (data?.done) {
      if (broadcastCompletion) {
        broadcastCompletion();
      }
    }
  }, [data]);

  return (
    <div className="grid w-full py-4 min-h-[180px] place-items-center">
      <div className="flex flex-col gap-1 items-center">
        <Text as="h2" variant="headingMd">
          {data?.done === true ? "Products Indexed!" : "Indexing products..."}
        </Text>
        <div className="opacity-30">{data?.chunkCount} Products</div>
        {!data?.done ? (
          <Spinner />
        ) : (
          <CheckIcon
            fill="#2A845A"
            color="#2A845A"
            style={{ height: "50px" }}
          />
        )}
      </div>
    </div>
  );
};
