import { createMemo, Show, useContext } from "solid-js";
import { JSX } from "solid-js";
import { DatasetContext } from "../contexts/DatasetContext";
import { A, useLocation } from "@solidjs/router";
import { AiOutlineLeft } from "solid-icons/ai";
import { Spacer } from "./Spacer";
import { Portal } from "solid-js/web";
import { NavbarDatasetSelector } from "../layouts/NavbarDatasetSelector";
import { FiExternalLink } from "solid-icons/fi";
import { UserContext } from "../contexts/UserContext";
import { IconTypes } from "solid-icons";

const apiHost = import.meta.env.VITE_API_HOST as string;
const analyticsUiURL = import.meta.env.VITE_ANALYTICS_UI_URL as string;
const searchUiURL = import.meta.env.VITE_SEARCH_UI_URL as string;
const chatUiURL = import.meta.env.VITE_CHAT_UI_URL as string;

export const DashboardSidebar = () => {
  const { datasetId } = useContext(DatasetContext);
  const userContext = useContext(UserContext);
  const pathname = useLocation();

  const orgDatasetParams = createMemo(() => {
    const orgId = userContext.selectedOrganization().id;
    let params = "";
    if (orgId) params += `?organization=${orgId}`;
    if (orgId && datasetId) params += `&dataset=${datasetId}`;
    return params;
  });

  const SectionLabel = (props: { children: string }) => (
    <div class="border-b border-b-neutral-200 text-sm text-neutral-500">
      {props.children}
    </div>
  );

  const Link = (props: {
    href: string;
    label: JSX.Element;
    isExternal: boolean;
    icon?: IconTypes;
  }) => (
    <A
      href={props.href}
      class="flex items-center justify-between gap-2 rounded-md p-1 px-2 hover:underline"
      classList={{
        "bg-magenta-200/30": pathname.pathname === props.href,
      }}
    >
      {props.label}
      <Show when={props.isExternal}>
        <FiExternalLink class="text-neutral-500" />
      </Show>
    </A>
  );

  return (
    <>
      {/* eslint-disable-next-line @typescript-eslint/no-non-null-assertion
       */}
      <Portal mount={document.querySelector("#dataset-slot")!}>
        <NavbarDatasetSelector />
      </Portal>
      <div class="border-r border-r-neutral-300 bg-neutral-50 px-4 pt-2">
        <A
          href="/"
          class="flex items-center gap-2 text-xs text-neutral-500 hover:underline"
        >
          <AiOutlineLeft size={12} />
          <div>Back to Organization</div>
        </A>
        <Spacer h={9} withBorder />
        <div class="pt-4">
          <div class="gap flex flex-col">
            <Link
              href={`/dataset/${datasetId}`}
              label="Overview"
              isExternal={false}
            />
            <Link
              href={`/dataset/${datasetId}/events`}
              label={"Event Log"}
              isExternal={false}
            />
          </div>
          <div class="gap flex flex-col pt-4">
            <SectionLabel>Dataset Settings</SectionLabel>
            <Link
              isExternal={false}
              href={`/dataset/${datasetId}/keys`}
              label="API Keys"
            />
            <Link
              isExternal={false}
              href={`/dataset/${datasetId}/llm-settings`}
              label="LLM Settings"
            />
            <Link
              isExternal={false}
              href={`/dataset/${datasetId}/options`}
              label="Dataset Options"
            />
            <Link
              isExternal={false}
              href={`/dataset/${datasetId}/manage`}
              label="Manage Dataset"
            />
          </div>
          <div class="gap flex flex-col pt-12">
            <SectionLabel>Playgrounds</SectionLabel>
            <Link
              isExternal={true}
              href={`${searchUiURL}${orgDatasetParams()}`}
              label="Search"
            />
            <Link
              isExternal={true}
              href={`${chatUiURL}${orgDatasetParams()}`}
              label="Chat"
            />
          </div>
        </div>
      </div>
    </>
  );
};
