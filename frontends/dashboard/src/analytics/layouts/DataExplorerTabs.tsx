import { A } from "@solidjs/router";
import { For, ParentComponent } from "solid-js";
import { useBetterNav } from "../utils/useBetterNav";
import { usePathname } from "../hooks/usePathname";

const pages: { name: string; path: string }[] = [
  {
    name: "Searches",
    path:
      window.location.pathname.split("/").slice(0, -1).join("/") + "/searches",
  },
  {
    name: "RAG Messages",
    path:
      window.location.pathname.split("/").slice(0, -1).join("/") + "/messages",
  },
];

export const DataExplorerTabs: ParentComponent = (props) => {
  const betterNav = useBetterNav();
  const pathname = usePathname();

  const handleLinkClick = (e: MouseEvent, path: string) => {
    e.preventDefault();
    betterNav(path);
  };

  return (
    <>
      <div class="flex gap-8 border-b border-b-neutral-200 px-2 pb-1">
        <For each={pages}>
          {(page) => (
            <A
              classList={{
                "font-medium": true,
                "text-fuchsia-800": pathname() === page.path,
              }}
              href={page.path}
              onClick={(e) => handleLinkClick(e, page.path)}
            >
              {page.name}
            </A>
          )}
        </For>
      </div>
      <div class="px-2 pt-2">{props.children}</div>
    </>
  );
};
