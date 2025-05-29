import { createMemo, Show, useContext } from "solid-js";
import { JSX } from "solid-js";
import { DatasetContext } from "../contexts/DatasetContext";
import { A, useLocation } from "@solidjs/router";
import {
  AiOutlineCamera,
  AiOutlineFilter,
  AiOutlineHistory,
  AiOutlineInfoCircle,
  AiOutlineKey,
  AiOutlineLeft,
  AiOutlineMessage,
  AiOutlineSearch,
  AiOutlineReload,
} from "solid-icons/ai";
import { Spacer } from "./Spacer";
import { Portal } from "solid-js/web";
import { NavbarDatasetSelector } from "../layouts/NavbarDatasetSelector";
import { NavbarOrganizationSelector } from "../layouts/NavbarOrganizationSelector";
import { FiExternalLink, FiTrash, FiShare2 } from "solid-icons/fi";
import { UserContext } from "../contexts/UserContext";
import { IconTypes } from "solid-icons";
import { IoOptionsOutline } from "solid-icons/io";
import { TbSparkles, TbTimelineEventText, TbTransform } from "solid-icons/tb";
import { createSignal } from "solid-js";
import NewDatasetModal from "../components/NewDatasetModal";
import { ImNewspaper } from "solid-icons/im";
import { useTrieve } from "../hooks/useTrieve";
import { createQuery } from "@tanstack/solid-query";

const searchUiURL =
  (import.meta.env.VITE_SEARCH_UI_URL as string | undefined) ?? "";
const chatUiURL =
  (import.meta.env.VITE_CHAT_UI_URL as string | undefined) ?? "";

export const DashboardSidebar = () => {
  const { datasetId } = useContext(DatasetContext);
  const userContext = useContext(UserContext);
  const pathname = useLocation();
  const trieve = useTrieve();

  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);

  const orgDatasetParams = createMemo(() => {
    const orgId = userContext.selectedOrg().id;
    let params = "";
    if (orgId) params += `?organization=${orgId}`;
    if (orgId && datasetId) params += `&dataset=${datasetId()}`;
    return params;
  });

  const SectionLabel = (props: { children: string }) => (
    <div class="border-b border-b-neutral-200 text-sm text-neutral-500">
      {props.children}
    </div>
  );

  const orgDatasets = createMemo(() => {
    const datasets = userContext.orgDatasets?.();
    return datasets || [];
  });

  const currentUserRole = createMemo(() => {
    return (
      userContext.user().user_orgs.find((val) => {
        return val.organization_id === userContext.selectedOrg().id;
      })?.role ?? 0
    );
  });

  const uploadStatusQuery = createQuery(() => ({
    queryKey: ["upload-status", datasetId()],
    queryFn: async () => {
      return await trieve.fetch(
        "/api/dataset/get_dataset_queue_lengths",
        "get",
        {
          datasetId: datasetId(),
        },
      );
    },
    refetchInterval: 5000,
    refetchOnMount: true,
    refetchOnWindowFocus: true,
    enabled: !!datasetId(),
  }));

  const UploadStatusBar = () => {
    return (
      <div class="space-y-2 p-2">
        <div class="flex flex-col justify-between">
          <div class="text-sm font-medium text-neutral-700">
            Processing Queue
          </div>
          <div class="flex items-center justify-between text-xs">
            <span class="text-neutral-500">Updates every 5 seconds</span>
            <button
              onClick={() => {
                void uploadStatusQuery.refetch();
              }}
              class="flex items-center gap-1 text-neutral-500 hover:text-fuchsia-500"
              title="Reload status"
            >
              <AiOutlineReload
                size={14}
                classList={{
                  "animate-spin": uploadStatusQuery.isFetching,
                }}
              />
            </button>
          </div>
        </div>

        <div class="space-y-1">
          <div class="flex justify-between text-xs text-neutral-600">
            <span>Files</span>
            <span>{uploadStatusQuery.data?.file_queue_length ?? 0} files</span>
          </div>
          <div class="h-2 w-full rounded-full bg-neutral-200">
            <div
              class="h-2 rounded-full bg-blue-500 transition-all duration-300"
              style={{
                width: `${Math.min(
                  ((uploadStatusQuery.data?.file_queue_length ?? 0) / 20) * 100,
                  100,
                )}%`,
              }}
            />
          </div>
        </div>

        <div class="space-y-1">
          <div class="flex justify-between text-xs text-neutral-600">
            <span>Chunk Batches</span>
            <span>
              {uploadStatusQuery.data?.chunk_queue_length ?? 0} batches
            </span>
          </div>
          <div class="h-2 w-full rounded-full bg-neutral-200">
            <div
              class="h-2 rounded-full bg-orange-500 transition-all duration-300"
              style={{
                width: `${Math.min(
                  ((uploadStatusQuery.data?.chunk_queue_length ?? 0) / 1000) *
                    100,
                  100,
                )}%`,
              }}
            />
          </div>
        </div>
      </div>
    );
  };

  const Link = (props: {
    href: string;
    label: JSX.Element;
    isExternal: boolean;
    icon?: IconTypes;
  }) => (
    <A
      href={props.href}
      target={props.isExternal ? "_blank" : undefined}
      class="flex items-center justify-between gap-2 rounded-md p-1 px-2 hover:underline"
      classList={{
        "bg-magenta-200/30": pathname.pathname === props.href,
      }}
    >
      <div class="flex items-center gap-2">
        <Show when={props.icon}>{(icon) => icon()({})}</Show>
        {props.label}
      </div>
      <Show when={props.isExternal}>
        <FiExternalLink class="text-neutral-500" />
      </Show>
    </A>
  );

  return (
    <>
      <Portal mount={document.body}>
        <NewDatasetModal
          isOpen={newDatasetModalOpen}
          closeModal={() => {
            setNewDatasetModalOpen(false);
          }}
        />
      </Portal>
      {/* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */}
      <Portal mount={document.querySelector("#organization-slot")!}>
        <div class="flex flex-row content-center items-center">
          <NavbarOrganizationSelector />
          <span class="ml-2 font-bold text-neutral-600">/</span>
        </div>
      </Portal>
      {/* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */}
      <Portal mount={document.querySelector("#dataset-slot")!}>
        <div class="ml-1 flex flex-row">
          <Show when={orgDatasets().length > 0}>
            <NavbarDatasetSelector />
          </Show>
          <Show when={orgDatasets().length == 0}>
            <button
              class="flex content-center items-center rounded bg-magenta-500 px-3 py-1 text-sm font-semibold text-white"
              onClick={() => setNewDatasetModalOpen(true)}
            >
              Create Dataset +
            </button>
          </Show>
        </div>
      </Portal>
      <div class="border-r border-r-neutral-300 bg-neutral-50 px-4 pt-2">
        <A
          href="/org"
          class="flex items-center gap-2 text-[12px] text-neutral-700 hover:underline"
        >
          <AiOutlineLeft size={12} />
          <div>Back to Organization</div>
        </A>
        <Spacer h={9} withBorder />
        <div class="pt-4">
          <div class="gap flex flex-col">
            <Link
              href={`/dataset/${datasetId()}`}
              label="Overview"
              icon={AiOutlineInfoCircle}
              isExternal={false}
            />
            <Link
              href={`/dataset/${datasetId()}/events`}
              icon={AiOutlineHistory}
              label={"Admin Event Log"}
              isExternal={false}
            />
            <Link
              isExternal={false}
              href={`/dataset/${datasetId()}/keys`}
              icon={AiOutlineKey}
              label="API Keys"
            />
          </div>
          <div class="gap flex flex-col pt-6">
            <SectionLabel>Playgrounds</SectionLabel>
            <Link
              isExternal={true}
              icon={AiOutlineSearch}
              href={`${searchUiURL}${orgDatasetParams()}`}
              label="Search"
            />
            <Link
              isExternal={true}
              icon={AiOutlineMessage}
              href={`${chatUiURL}${orgDatasetParams()}`}
              label="Chat"
            />
          </div>
          <div class="gap flex flex-col pt-4">
            <SectionLabel>Analytics</SectionLabel>
            <Link
              isExternal={false}
              icon={ImNewspaper}
              href={`/dataset/${datasetId()}/analytics`}
              label="Overview"
            />
            <Link
              isExternal={false}
              icon={AiOutlineSearch}
              href={`/dataset/${datasetId()}/analytics/data/searches`}
              label="Searches"
            />
            <Link
              isExternal={false}
              icon={AiOutlineMessage}
              href={`/dataset/${datasetId()}/analytics/data/messages`}
              label="RAG Messages"
            />
            <Link
              isExternal={false}
              icon={AiOutlineFilter}
              href={`/dataset/${datasetId()}/analytics/data/recommendations`}
              label="Recommendations"
            />
            <Link
              isExternal={false}
              icon={TbTimelineEventText}
              href={`/dataset/${datasetId()}/analytics/data/events`}
              label="Events"
            />
          </div>
          <div class="gap flex flex-col pt-4">
            <SectionLabel>Dataset Settings</SectionLabel>
            <Link
              isExternal={false}
              icon={AiOutlineCamera}
              href={`/dataset/${datasetId()}/crawl/create`}
              label="Crawling Options"
            />
            <Link
              isExternal={false}
              icon={FiShare2}
              href={`/dataset/${datasetId()}/public-page`}
              label="Demo Page"
            />
            <Link
              isExternal={false}
              icon={TbTransform}
              href={`/dataset/${datasetId()}/batch-transform`}
              label="Batch Transform"
            />
            <Link
              isExternal={false}
              icon={TbSparkles}
              href={`/dataset/${datasetId()}/llm-settings`}
              label="LLM Options"
            />
            <Link
              isExternal={false}
              icon={IoOptionsOutline}
              href={`/dataset/${datasetId()}/options`}
              label="Dataset Options"
            />
            <Show when={currentUserRole() === 2}>
              <Link
                isExternal={false}
                icon={FiTrash}
                href={`/dataset/${datasetId()}/manage`}
                label="Manage Dataset"
              />
            </Show>
          </div>
          <div class="gap flex flex-col pt-4">
            <SectionLabel>Upload Status</SectionLabel>
            <UploadStatusBar />
          </div>
        </div>
      </div>
    </>
  );
};
