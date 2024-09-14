import { createMemo, For, useContext } from "solid-js";
import { JSX } from "solid-js";
import { DatasetContext } from "../contexts/DatasetContext";
import { A, useLocation } from "@solidjs/router";
import { AiOutlineLeft } from "solid-icons/ai";
import { Spacer } from "./Spacer";
import { Portal } from "solid-js/web";
import { NavbarDatasetSelector } from "../layouts/NavbarDatasetSelector";

type Link = {
  label: string;
  href: string;
  isExternal: boolean;
  icon?: JSX.Element;
};

export const DashboardSidebar = () => {
  const { datasetId } = useContext(DatasetContext);
  const pathname = useLocation();

  const datasetLinks = createMemo<Link[]>(() => [
    {
      href: `/dataset/${datasetId}`,
      label: "Overview",
      isExternal: false,
      // icon: <AiOutlineLeft size={12} />,
    },
    {
      href: `/dataset/${datasetId}/keys`,
      isExternal: false,
      label: "Keys",
      // icon: <AiOutlineLeft size={12} />,
    },
  ]);

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
          <div class="flex flex-col gap-2">
            <For each={datasetLinks()}>
              {(link) => (
                <A
                  href={link.href}
                  class="flex items-center gap-2 rounded-md p-2 hover:underline"
                  classList={{
                    "bg-magenta-200/30": pathname.pathname === link.href,
                  }}
                >
                  {link.icon}
                  <div>{link.label}</div>
                </A>
              )}
            </For>
          </div>
        </div>
      </div>
    </>
  );
};
