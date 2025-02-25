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
            searchPageProps={{
              filterSidebarProps: {
                sections: [
                  {
                    key: "categories",
                    title: "Categories",
                    options: [
                      { label: "Carpets", tag: "carpets" },
                      { label: "Flooring", tag: "flooring" },
                      { label: "Paint", tag: "paint" },
                      { label: "Countertops", tag: "countertop" },
                    ],
                    selectionType: "single",
                    filterType: "match_any",
                  },
                  {
                    key: "brands",
                    title: "Material",
                    options: [
                      { label: "Nylon", tag: "nylon" },
                      { label: "Quartz", tag: "quartz" },
                      { label: "Hardwood", tag: "hardwood" },
                      { label: "Laminate", tag: "laminate" },
                    ],
                    selectionType: "multiple",
                    filterType: "match_any",
                  },
                ],
              },
              inferenceFiltersFormProps: {
                steps: [
                  {
                    title: "Upload Image",
                    description:
                      "Upload an image of the space you want to renovate or materials you like and we will recommend products that match your style.",
                    type: "image",
                    placeholder: "Click or drag to upload (Max 5MB)",
                  },
                  {
                    title: "Category Selection",
                    description:
                      "Select the materials you are interested in replacing.",
                    type: "tags",
                    placeholder: "Select categories",
                    filterSidebarSectionKey: "categories",
                  },
                ],
              },
              display: true,
            }}
          />
        </div>
      </div>
    </>
  );
}
