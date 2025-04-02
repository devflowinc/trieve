import { Spinner, Text } from "@shopify/polaris";
import { CheckIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { shopifyProductCountQuery } from "app/queries/onboarding";
import { usageQuery } from "app/queries/usage";
import { OnboardingBody } from "app/utils/onboarding";
import { useEffect, useState } from "react";

export const WelcomeOnboarding: OnboardingBody = ({
  broadcastCompletion,
  goToNextStep,
}) => {
  const adminApi = useClientAdminApi();
  const { trieve } = useTrieve();
  const [completed, setCompleted] = useState(false);
  const [refetch, setRefetch] = useState(true);

  const { data: usage } = useQuery({
    ...usageQuery(trieve),
    refetchInterval: 1000,
    enabled: refetch,
  });
  const { data: productCount } = useQuery(shopifyProductCountQuery(adminApi));

  useEffect(() => {
    if (!productCount || !usage?.chunk_count) {
      return;
    }
    if (usage?.chunk_count >= productCount) {
      setCompleted(true);
      setRefetch(false);
      if (broadcastCompletion) {
        broadcastCompletion();
      }
      if (goToNextStep) {
        goToNextStep();
      }
    }
  }, [usage, productCount]);

  return (
    <div className="grid w-full py-4 min-h-[180px] place-items-center">
      <div className="flex flex-col gap-1 items-center">
        <Text as="h2" variant="headingMd">
          {completed === true ? "Products Indexed!" : "Indexing products..."}
        </Text>
        <div className="opacity-30">{usage?.chunk_count} Products</div>
        {!completed ? (
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
