import { TrieveModalSearch } from "../../../src/index";
import "../../../dist/index.css";
import { useState } from "react";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/searchpage")({
  component: ECommerce,
});

export default function ECommerce() {
  const baseUrl = import.meta.env.VITE_API_BASE_URL;
  const datasetId = import.meta.env.VITE_DATASET_ID;
  const apiKey = import.meta.env.VITE_API_KEY;

  const [theme, setTheme] = useState<"light" | "dark">("light");

  return (
    <>
      <div
        className={`min-w-screen min-h-screen relative ${
          theme === "dark" ? "bg-zinc-900 text-zinc-50" : ""
        }`}
      >
        <div className="absolute top-6 right-6">
          <ul>
            <li key="theme">
              <button
                onClick={() => setTheme(theme === "light" ? "dark" : "light")}
              >
                {theme === "light" ? (
                  <span>
                    <i className="fa-regular fa-sun"></i>
                  </span>
                ) : (
                  <span>
                    <i className="fa-regular fa-moon"></i>
                  </span>
                )}
              </button>
            </li>
          </ul>
        </div>
        <div className="w-full">
          <TrieveModalSearch
            displayModal={false}
            datasetId={datasetId}
            apiKey={apiKey}
            baseUrl={baseUrl}
            brandColor="#000"
            filterSidebarProps={{
              sections: [
                {
                  key: "categories",
                  title: "Categories",
                  options: [
                    { label: "All", tag: "all" },
                    { label: "Clothing", tag: "clothing" },
                    { label: "Shoes", tag: "shoes" },
                    { label: "Accessories", tag: "accessories" },
                  ],
                  selectionType: "single",
                  filterType: "match_any",
                },
                {
                  key: "brands",
                  title: "Brands",
                  options: [
                    { label: "All", tag: "all" },
                    { label: "Nike", tag: "nike" },
                    { label: "Adidas", tag: "adidas" },
                    { label: "Puma", tag: "puma" },
                    { label: "Reebok", tag: "reebok" },
                  ],
                  selectionType: "multiple",
                  filterType: "match_any",
                },
                {
                  key: "price",
                  title: "Price Range",
                  options: [
                    { label: "$0 - $50", tag: "0-50" },
                    { label: "$50 - $100", tag: "50-100" },
                    { label: "$100 - $150", tag: "100-150" },
                    { label: "$150+", tag: "150+" },
                  ],
                  selectionType: "single",
                  filterType: "match_any",
                },
              ],
              display: true,
            }}
          />
        </div>
      </div>
    </>
  );
}
