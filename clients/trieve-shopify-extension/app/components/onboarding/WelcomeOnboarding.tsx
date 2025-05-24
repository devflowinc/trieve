import { Spinner, Text } from "@shopify/polaris";
import { CheckIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { trackCustomerEvent } from "app/processors/shopifyTrackers";
import { shopifyVariantsCountQuery } from "app/queries/onboarding";
import { datasetUsageQuery } from "app/queries/usage";
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
    ...datasetUsageQuery(trieve),
    refetchInterval: 1000,
    enabled: refetch,
  });
  const { data: productVariantsCount } = useQuery(
    shopifyVariantsCountQuery(adminApi),
  );

  useEffect(() => {
    if (!productVariantsCount || !usage?.chunk_count) {
      return;
    }
    if (usage?.chunk_count >= productVariantsCount * 0.1) {
      setCompleted(true);
      setRefetch(false);
      if (trieve.organizationId && trieve.trieve.apiKey != null) {
        trackCustomerEvent(
          trieve.trieve.baseUrl,
          {
            organization_id: trieve.organizationId,
            store_name: "",
            event_type: "catalogue_indexed",
          },
          trieve.organizationId,
          trieve.trieve.apiKey,
        );
      }
      broadcastCompletion();
      if (goToNextStep) {
        goToNextStep();
      }
    }
  }, [usage, productVariantsCount]);

  return (
    <div className="grid w-full py-4 min-h-[180px] place-items-center">
      <div className="flex flex-col gap-1 items-center">
        <Text as="h2" variant="headingMd">
          {completed === true ? "Products Indexed!" : "Indexing products..."}
        </Text>
        <div className="opacity-30">
          Ingested {usage?.chunk_count} out of {productVariantsCount} products
        </div>
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
