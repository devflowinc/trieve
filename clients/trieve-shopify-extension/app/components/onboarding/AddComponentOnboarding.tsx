import { Button, SkeletonBodyText, Text } from "@shopify/polaris";
import { CheckIcon, ExternalIcon } from "@shopify/polaris-icons";
import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { useClientAdminApi } from "app/loaders/clientLoader";
import {
  singleThemeGlobalComponentInstallQuery,
  singleThemePdpComponentInstallQuery,
  themeListQuery,
} from "app/queries/onboarding";
import { cn } from "app/utils/cn";
import { OnboardingBody } from "app/utils/onboarding";
import { useState } from "react";
import { ThemeSelect, ThemeChoice } from "./ThemeSelect";
import {
  useAddComponentDeepLink,
  useAddComponentOnboarding,
} from "app/hooks/add-component-onboard";
import { TutorialVideo } from "./TutorialVideo";
import { withSuspense } from "app/utils/suspense";

export const AddComponentOnboarding: OnboardingBody = withSuspense(
  ({ broadcastCompletion }) => {
    const adminApi = useClientAdminApi();

    useAddComponentOnboarding(broadcastCompletion);

    const { data: themes } = useSuspenseQuery({
      ...themeListQuery(adminApi),
    });

    const [selectedTheme, setSelectedTheme] = useState<ThemeChoice>(themes[0]);

    const { data: globalInstalledOnSelectedTheme } = useQuery(
      singleThemeGlobalComponentInstallQuery(adminApi, selectedTheme.id),
    );

    const { data: pdpInstalledOnSelectedTheme } = useQuery(
      singleThemePdpComponentInstallQuery(adminApi, selectedTheme.id),
    );

    const { getDeeplink, getPdpDeepLink } =
      useAddComponentDeepLink(selectedTheme);

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

    const handleThemeChange = (theme: ThemeChoice) => {
      setSelectedTheme(theme);
    };

    return (
      <div>
        <div className="px-5 mb-2 max-w-[300px]">
          <ThemeSelect
            themes={themes}
            selectedTheme={selectedTheme}
            onChange={handleThemeChange}
            disabled={themes.length < 2}
          />
        </div>
        <div className={cn("grid w-full pb-1 place-items-center")}>
          <div className="grid grid-cols-2 pb-4 pt-4 px-8 w-full">
            <div className="flex flex-col gap-1 border-r border-r-neutral-200 items-center">
              <div className="flex items-center gap-1">
                <Text as="h2" alignment="center" variant="headingMd">
                  {globalInstalledOnSelectedTheme === true
                    ? "Global Search"
                    : "Add the global search component"}
                </Text>
                {globalInstalledOnSelectedTheme && (
                  <CheckIcon
                    fill="#2A845A"
                    color="#2A845A"
                    style={{ height: "20px" }}
                  />
                )}
              </div>
              {getDeeplink() && (
                <div className="flex flex-col gap-2">
                  <Button onClick={openDeepLink}>
                    {globalInstalledOnSelectedTheme ? "Edit on" : "Add to"}{" "}
                    {selectedTheme.name}
                  </Button>
                  <TutorialVideo
                    title="Add The Global Search Component"
                    url="https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                    editStoreButton={
                      <Button icon={ExternalIcon} onClick={openDeepLink}>
                        {globalInstalledOnSelectedTheme ? "Edit on" : "Add to"}{" "}
                        {selectedTheme.name}
                      </Button>
                    }
                  />
                </div>
              )}
            </div>
            <div className="flex flex-col gap-1 items-center">
              <div className="flex items-center gap-1">
                <Text as="h2" alignment="center" variant="headingMd">
                  {pdpInstalledOnSelectedTheme === true
                    ? "Product Chat"
                    : "Add the product chat component"}
                </Text>
                {pdpInstalledOnSelectedTheme && (
                  <CheckIcon
                    fill="#2A845A"
                    color="#2A845A"
                    style={{ height: "20px" }}
                  />
                )}
              </div>
              {getPdpDeepLink() && (
                <div className="flex flex-col gap-2">
                  <Button onClick={openPdpDeepLink}>
                    {pdpInstalledOnSelectedTheme ? "Edit on" : "Add to"}{" "}
                    {selectedTheme.name}
                  </Button>
                  <TutorialVideo
                    title="Add The Product Chat Component"
                    url="https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                    editStoreButton={
                      <Button icon={ExternalIcon} onClick={openPdpDeepLink}>
                        {pdpInstalledOnSelectedTheme ? "Edit on" : "Add to"}{" "}
                        {selectedTheme.name}
                      </Button>
                    }
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  },

  <div className="h-[180px] px-2">
    <SkeletonBodyText lines={8} />
  </div>,
);
