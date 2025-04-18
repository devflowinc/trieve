import { Button, Text } from "@shopify/polaris";
import { CheckIcon, ExternalIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { themeListQuery } from "app/queries/onboarding";
import { cn } from "app/utils/cn";
import { OnboardingBody } from "app/utils/onboarding";
import { useEffect, useState } from "react";
import { ThemeSelect, ThemeChoice } from "./ThemeSelect";
import {
  useAddComponentDeepLink,
  useAddComponentOnboarding,
} from "app/hooks/add-component-onboard";
import { TutorialVideo } from "./TutorialVideo";

export const AddComponentOnboarding: OnboardingBody = ({
  broadcastCompletion,
}) => {
  const adminApi = useClientAdminApi();

  const { allDoneGlobally, globalComplete, pdpComplete } =
    useAddComponentOnboarding(broadcastCompletion);

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

  const themeName = selectedTheme?.name ? `"${selectedTheme?.name}"` : "...";

  const handleThemeChange = (theme: ThemeChoice) => {
    setSelectedTheme(theme);
  };

  return (
    <div className={cn(!allDoneGlobally && "min-h-[180px]")}>
      <div className="px-5 mb-2 max-w-[300px]">
        <ThemeSelect
          themes={themes}
          selectedTheme={selectedTheme}
          onChange={handleThemeChange}
          disabled={themes.length < 2}
        />
      </div>
      <div className={cn("grid w-full pb-1 place-items-center")}>
        <div className="grid grid-cols-2 pt-4 px-8 w-full">
          <div className="flex flex-col gap-1 border-r border-r-neutral-200 items-center">
            <Text as="h2" alignment="center" variant="headingMd">
              {globalComplete === true
                ? "Global Search"
                : "Add the global search component"}
            </Text>
            {!globalComplete && getDeeplink() && (
              <div className="flex flex-col gap-2">
                <Button onClick={openDeepLink}>Add to {themeName}</Button>
                <TutorialVideo
                  title="Add The Global Search Component"
                  url="https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                  editStoreButton={
                    <Button icon={ExternalIcon} onClick={openDeepLink}>
                      Add to {themeName}
                    </Button>
                  }
                />
              </div>
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
            <Text as="h2" alignment="center" variant="headingMd">
              {pdpComplete === true
                ? "Product Chat"
                : "Add the product chat component"}
            </Text>
            {!pdpComplete && getPdpDeepLink() && (
              <div className="flex flex-col gap-2">
                <Button onClick={openPdpDeepLink}>Add to {themeName}</Button>
                <TutorialVideo
                  title="Add The Product Chat Component"
                  url="https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                  editStoreButton={
                    <Button icon={ExternalIcon} onClick={openPdpDeepLink}>
                      Add to {themeName}
                    </Button>
                  }
                />
              </div>
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
