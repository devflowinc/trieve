import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { trackCustomerEvent } from "app/processors/shopifyTrackers";
import {
  globalComponentInstallQuery,
  pdpInstallQuery,
} from "app/queries/onboarding";
import { useEffect, useMemo, useState } from "react";

export const useAddComponentOnboarding = (broadcastCompletion: () => void) => {
  const { trieve } = useTrieve();
  const adminApi = useClientAdminApi();
  // Track global install
  const [keepFetchingGlobal, setKeepFetchingGlobal] = useState(true);
  const [keepFetchingPdp, setKeepFetchingPdp] = useState(true);

  const { data: globalThemeData } = useQuery({
    ...globalComponentInstallQuery(adminApi),
    refetchInterval: 2000,
    placeholderData: {},
    enabled: keepFetchingGlobal,
  });

  const globalComplete = useMemo(() => {
    const stringified = JSON.stringify(globalThemeData);
    if (stringified?.includes("global_component")) return true;
    return false;
  }, [globalThemeData]);

  const { data: pdpThemeData } = useQuery({
    ...pdpInstallQuery(adminApi),
    refetchInterval: 2000,
    placeholderData: {},
    enabled: keepFetchingPdp,
  });

  const pdpComplete = useMemo(() => {
    const stringified = JSON.stringify(pdpThemeData);
    if (stringified?.includes("inline_component")) return true;
    return false;
  }, [pdpThemeData]);

  useEffect(() => {
    if (globalComplete) {
      setKeepFetchingGlobal(false);
    }
    if (pdpComplete) {
      setKeepFetchingPdp(false);
    }
  }, [globalComplete, pdpComplete]);

  const allDone = globalComplete && pdpComplete;

  useEffect(() => {
    if (globalComplete) {
      if (trieve.organizationId && trieve.trieve.apiKey != null) {
        trackCustomerEvent(
          trieve.trieve.baseUrl,
          {
            organization_id: trieve.organizationId,
            store_name: "",
            event_type: "global_component_added",
          },
          trieve.organizationId,
          trieve.trieve.apiKey,
        );
      }
    }

    if (pdpComplete) {
      if (trieve.organizationId && trieve.trieve.apiKey != null) {
        trackCustomerEvent(
          trieve.trieve.baseUrl,
          {
            organization_id: trieve.organizationId,
            store_name: "",
            event_type: "pdp_component_added",
          },
          trieve.organizationId,
          trieve.trieve.apiKey,
        );
      }
    }

    if (allDone) {
      broadcastCompletion();
    }
  }, [allDone]);

  return { allDoneGlobally: allDone, globalComplete, pdpComplete };
};
