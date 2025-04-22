import { useRouteLoaderData } from "@remix-run/react";
import { useQuery } from "@tanstack/react-query";
import { ThemeChoice } from "app/components/onboarding/ThemeSelect";
import { useTrieve } from "app/context/trieveContext";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { trackCustomerEvent } from "app/processors/shopifyTrackers";
import {
  globalComponentInstallQuery,
  pdpInstallQuery,
} from "app/queries/onboarding";
import { useShopName } from "app/utils/useShopName";
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

export const getShortThemeId = (fullGid: string): string | null => {
  const regex = /gid:\/\/shopify\/OnlineStoreTheme\/(\d+)/;
  const match = fullGid.match(regex);
  if (match && match.length > 1) {
    return match[1];
  }
  return null;
};

export const useAddComponentDeepLink = (theme: ThemeChoice) => {
  const shopname = useShopName();

  const { shopifyThemeAppExtensionUuid } = useRouteLoaderData("routes/app") as {
    shopifyThemeAppExtensionUuid?: string;
  };

  const getDeeplink = () => {
    if (!shopifyThemeAppExtensionUuid) return null;
    if (!shopname) return null;
    const themeId = getShortThemeId(theme.id);

    return `https://${shopname}/admin/themes/${themeId}/editor?context=apps&activateAppId=${shopifyThemeAppExtensionUuid}/global_component`;
  };

  const getPdpDeepLink = () => {
    if (!shopifyThemeAppExtensionUuid) return null;
    if (!shopname) return null;
    const themeId = getShortThemeId(theme.id);

    return `https://${shopname}/admin/themes/${themeId}/editor?template=product&addAppBlockId=${shopifyThemeAppExtensionUuid}/inline_component`;
  };

  return { getDeeplink, getPdpDeepLink };
};
