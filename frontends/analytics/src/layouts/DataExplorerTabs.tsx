import { A } from "@solidjs/router";
import { For, ParentComponent } from "solid-js";
import { useBetterNav } from "../utils/useBetterNav";
import { usePathname } from "../hooks/usePathname";

const pages: { name: string; path: string }[] = [
  {
    name: "Searches",
    path: "/data/searches",
  },
  {
    name: "RAG Messages",
    path: "/data/messages",
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
      <div class="flex gap-8 border-b-2 border-b-neutral-300 px-2 pb-1">
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
