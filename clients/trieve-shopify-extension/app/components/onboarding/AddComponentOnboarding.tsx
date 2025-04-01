import { useRouteLoaderData } from "@remix-run/react";
import { Button, Select, Text } from "@shopify/polaris";
import { CheckIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useClientAdminApi } from "app/loaders/clientLoader";
import {
  themeListQuery,
  globalComponentInstallQuery,
  pdpInstallQuery,
} from "app/queries/onboarding";
import { cn } from "app/utils/cn";
import { OnboardingBody } from "app/utils/onboarding";
import { useShopName } from "app/utils/useShopName";
import { useEffect, useMemo, useState } from "react";

type ThemeChoice = {
  name: string;
  prefix: string;
  id: string;
  role: string;
  updatedAt: string;
};

const getShortThemeId = (fullGid: string): string | null => {
  const regex = /gid:\/\/shopify\/OnlineStoreTheme\/(\d+)/;
  const match = fullGid.match(regex);
  if (match && match.length > 1) {
    return match[1];
  }
  return null;
};

export const AddComponentOnboarding: OnboardingBody = ({
  broadcastCompletion,
}) => {
  const adminApi = useClientAdminApi();

  const [keepFetchingGlobal, setKeepFetchingGlobal] = useState(true);

  const { data: globalThemeData } = useQuery({
    ...globalComponentInstallQuery(adminApi),
    refetchInterval: 2000,
    placeholderData: {},
    enabled: keepFetchingGlobal,
  });

  const [keepFetchingPdp, setKeepFetchingPdp] = useState(true);

  const { data: pdpThemeData } = useQuery({
    ...pdpInstallQuery(adminApi),
    refetchInterval: 2000,
    placeholderData: {},
    enabled: keepFetchingPdp,
  });

  const { data: themes } = useQuery({
    ...themeListQuery(adminApi),
    initialData: [],
    staleTime: 0,
  });

  const [selectedTheme, setSelectedTheme] = useState<ThemeChoice | null>(null);

  // Auto selects first theme if none selected
  useEffect(() => {
    if (themes.length > 0 && selectedTheme === null) {
      setSelectedTheme(themes[0]);
    }
  }, [themes, selectedTheme]);

  const globalComplete = useMemo(() => {
    const stringified = JSON.stringify(globalThemeData);
    if (stringified?.includes("trieve.ai")) return true;
    return false;
  }, [globalThemeData]);

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

  const shopname = useShopName();

  const { shopifyThemeAppExtensionUuid } = useRouteLoaderData("routes/app") as {
    shopifyThemeAppExtensionUuid?: string;
  };

  const getDeeplink = () => {
    if (!shopifyThemeAppExtensionUuid) return null;
    if (!shopname) return null;
    if (!selectedTheme) return null;
    const themeId = getShortThemeId(selectedTheme?.id);

    return `https://${shopname}/admin/themes/${themeId}/editor?context=apps&activateAppId=${shopifyThemeAppExtensionUuid}/global_component`;
  };

  const getPdpDeepLink = () => {
    if (!shopifyThemeAppExtensionUuid) return null;
    if (!shopname) return null;
    if (!selectedTheme) return null;
    const themeId = getShortThemeId(selectedTheme?.id);

    return `https://${shopname}/admin/themes/${themeId}/editor?template=product&addAppBlockId=${shopifyThemeAppExtensionUuid}/inline_component`;
  };

  const openDeepLink = () => {
    const link = getDeeplink();
    if (!link) return;
    window.open(link, "_blank");
  };

  const openPdpDeepLink = () => {
    const link = getPdpDeepLink();
    if (!link) return;
    window.open(link, "_blank");
  };

  useEffect(() => {
    if (globalComplete && pdpComplete) {
      if (broadcastCompletion) {
        broadcastCompletion();
      }
    }
  }, [globalComplete, pdpComplete]);

  const themeName = selectedTheme?.name ? `"${selectedTheme?.name}"` : "...";

  const allDone = globalComplete && pdpComplete;

  return (
    <div className={cn(!allDone && "min-h-[180px]")}>
      {!allDone && (
        <div className="px-5 mb-2 max-w-[300px]">
          <Select
            disabled={themes.length < 2}
            label="Theme"
            value={selectedTheme?.name}
            onChange={(e) => {
              setSelectedTheme(themes.find((t) => t.name === e)!);
            }}
            options={themes.map((t) => ({
              label: t.name,
              value: t.name,
            }))}
          ></Select>
        </div>
      )}
      <div
        className={cn(
          "grid w-full pb-1 place-items-center",
          allDone && "min-h-[180px]",
        )}
      >
        <div className="grid grid-cols-2 pt-4 px-8 w-full">
          <div className="flex flex-col gap-1 border-r border-r-neutral-200 items-center">
            <Text as="h2" variant="headingMd">
              {globalComplete === true
                ? "Global Search"
                : "Add the global search component"}
            </Text>
            {!globalComplete && getDeeplink() && (
              <Button onClick={openDeepLink}>Add to {themeName}</Button>
            )}
            {globalComplete && (
              <CheckIcon
                fill="#2A845A"
                color="#2A845A"
                style={{ height: "50px" }}
              />
            )}
          </div>
          <div className="flex flex-col gap-1 items-center">
            <Text as="h2" variant="headingMd">
              {pdpComplete === true
                ? "Product Chat"
                : "Add the product chat component"}
            </Text>
            {!pdpComplete && getPdpDeepLink() && (
              <Button onClick={openPdpDeepLink}>Add to {themeName}</Button>
            )}
            {pdpComplete && (
              <CheckIcon
                fill="#2A845A"
                color="#2A845A"
                style={{ height: "50px" }}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
