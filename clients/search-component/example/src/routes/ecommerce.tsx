import { TrieveModalSearch } from "../../../src/index";
import "../../../dist/index.css";
import { useState } from "react";
import { IconMoon, IconNext, IconPrevious, IconSun } from "../Icons";

import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/ecommerce")({
  component: ECommerce,
});

export default function ECommerce() {
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const [component, setComponent] = useState(0);
  return (
    <>
      <div
        className={`p-12 flex flex-col items-center justify-center w-screen h-screen relative ${
          theme === "dark" ? "bg-zinc-900 text-zinc-50" : ""
        }`}
      >
        <div className="absolute top-6 right-6">
          <ul>
            <li key="theme">
              <button
                onClick={() => setTheme(theme === "light" ? "dark" : "light")}
              >
                {theme === "light" ? <IconMoon /> : <IconSun />}
              </button>
            </li>
          </ul>
        </div>
        {component === 0 ? (
          <>
            <h2 className="font-bold text-center py-8">
              Search Modal Component{" "}
            </h2>

            <TrieveModalSearch
              type="ecommerce"
              defaultSearchMode="search"
              apiKey="tr-IzHXaZ89aELJXk0RkP1oIgqA6GFLGInG"
              datasetId="78c10935-12ad-4431-bb0f-3d9929262861"
              brandLogoImgSrcUrl="https://www.zedlabz.com/cdn/shop/t/2/assets/zedlabz-logo-retina.png?v=156539475513264331071650193684"
              brandName="ZedLabs"
              brandColor="#204364"
              defaultCurrency="$"
              currencyPosition="before"
            />
          </>
        ) : (
          <>
            <h2 className="font-bold text-center py-8">
              Search Results Component
            </h2>
            <h2 className="font-bold text-center py-8">
              This was removed, see
              https://github.com/devflowinc/trieve/pull/2613
            </h2>
          </>
        )}

        <ul className="absolute top-1/2 -translate-y-1/2 w-full">
          {component > 0 ? (
            <li className="left-6 absolute">
              <button onClick={() => setComponent(0)}>
                <IconPrevious />
              </button>
            </li>
          ) : (
            <li className="right-6 absolute">
              <button onClick={() => setComponent(1)}>
                <IconNext />
              </button>
            </li>
          )}
        </ul>
      </div>
    </>
  );
}
