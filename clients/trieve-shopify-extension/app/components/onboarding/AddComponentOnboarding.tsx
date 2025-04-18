import { Button, Text } from "@shopify/polaris";
import { createPortal } from "react-dom";
import { CheckIcon, ExternalIcon, XIcon } from "@shopify/polaris-icons";
import { useQuery } from "@tanstack/react-query";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { themeListQuery } from "app/queries/onboarding";
import { cn } from "app/utils/cn";
import { OnboardingBody } from "app/utils/onboarding";
import { ReactNode, useEffect, useState } from "react";
import { ThemeSelect, ThemeChoice } from "./ThemeSelect";
import {
  useAddComponentDeepLink,
  useAddComponentOnboarding,
} from "app/hooks/add-component-onboard";

interface TutorialVideoProps {
  title: string;
  url: string;
  editStoreButton?: ReactNode;
}
const TutorialVideo = (props: TutorialVideoProps) => {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button className="opacity-80" onClick={() => setOpen(true)}>
        Watch Tutorial
      </button>
      {open &&
        createPortal(
          <div
            onClick={() => setOpen(false)}
            className="bg-neutral-800/20 h-full w-full fixed top-0 left-0 z-[800]"
          >
            <div className="flex flex-col items-center justify-center h-full z-[900] w-full">
              <div className="w-full h-full bg-neutral-800/10 flex flex-col items-center justify-center">
                <div
                  onClick={(e) => e.stopPropagation()}
                  className="bg-white shadow rounded-lg overflow-hidden"
                >
                  <div className="p-2 pl-4 py-3 font-semibold flex justify-between items-center">
                    {props.title}
                    <div className="flex items-center gap-4">
                      {props.editStoreButton}
                      <button onClick={() => setOpen(false)} className="p-1">
                        <XIcon width={23} height={23}></XIcon>
                      </button>
                    </div>
                  </div>
                  <iframe
                    className="w-[80vw] aspect-video"
                    src="https://www.youtube.com/embed/_FUHj3XF8O0?si=1WCOl7evpyH5j4WY"
                    title="YouTube video player"
                    frameBorder="0"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                    referrerPolicy="strict-origin-when-cross-origin"
                    allowFullScreen
                  ></iframe>
                </div>
              </div>
            </div>
          </div>,
          document.body,
        )}
    </>
  );
};

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
            <Text as="h2" variant="headingMd">
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
            <Text as="h2" variant="headingMd">
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
