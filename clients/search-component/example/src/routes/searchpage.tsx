import "../../../dist/index.css";
import "../custom-css/searchpage.css";
import { TrieveModalSearch } from "../../../src/index";
import { createFileRoute } from "@tanstack/react-router";
import { TagProp } from "../../../src/utils/hooks/modal-context";

export const Route = createFileRoute("/searchpage")({
  component: ECommerce,
});

const options: TagProp[] = [
  {
    label: "Wallcovering",
    tag: "Wallcovering",
    child: {
      key: "wallcoveringChild",
      title: "Subcategory (Optional)",
      selectionType: "single",
      filterType: "match_all",
      options: [
        {
          label: "Natural",
          tag: "Natural",
        },
        {
          label: "Paper",
          tag: "Paper",
        },
        {
          label: "Synthetic",
          tag: "Synthetic",
        },
        {
          label: "Vinyl",
          tag: "Vinyl",
        },
      ],
    },
  },
  {
    label: "Countertops",
    tag: "Countertops",
    child: {
      key: "countertopsChild",
      title: "Subcategory (Optional)",
      selectionType: "single",
      filterType: "match_all",
      options: [
        {
          label: "Engineered & Composite",
          tag: "Engineered & Composite",
        },
        {
          label: "Laminate",
          tag: "Laminate",
        },
        {
          label: "Solid Surface",
          tag: "Solid Surface",
        },
        {
          label: "Stone",
          tag: "Stone",
        },
      ],
    },
  },
  {
    label: "Tile",
    tag: "Tile",
    child: {
      key: "tileChild",
      title: "Subcategory (Optional)",
      selectionType: "single",
      filterType: "match_all",
      options: [
        {
          label: "Backsplash",
          tag: "Backsplash",
        },
        {
          label: "Shower Flooring",
          tag: "Shower Flooring",
        },
        {
          label: "Shower Wall",
          tag: "Shower Wall",
        },
        {
          label: "Bathroom Flooring",
          tag: "Bathroom Flooring",
        },
        {
          label: "Bathroom Wall",
          tag: "Bathroom Wall",
        },
      ],
    },
  },
  {
    label: "Flooring",
    tag: "Flooring",
    child: {
      key: "flooringChild",
      title: "Subcategory (Optional)",
      selectionType: "single",
      filterType: "match_all",
      options: [
        {
          label: "Laminate",
          tag: "Laminate",
        },
        {
          label: "Luxury Vinyl Tile",
          tag: "Luxury Vinyl Tile",
        },
        {
          label: "Wood",
          tag: "Wood",
        },
      ],
    },
  },
  {
    label: "Paint",
    tag: "Paint",
  },
  {
    label: "Carpet",
    tag: "Carpet",
  },
  {
    label: "Fabric",
    tag: "Fabric",
    child: {
      key: "fabricChild",
      title: "Subcategory (Optional)",
      selectionType: "single",
      filterType: "match_all",
      options: [
        {
          label: "Cotton",
          tag: "Cotton",
        },
        {
          label: "Leather",
          tag: "Leather",
        },
        {
          label: "Linen",
          tag: "Linen",
        },
        {
          label: "Sheer",
          tag: "Sheer",
        },
        {
          label: "Synthetic",
          tag: "Synthetic",
        },
        {
          label: "Velvet",
          tag: "Velvet|DesignShop",
        },
        {
          label: "Wool",
          tag: "Wool",
        },
      ],
    },
  },
  {
    label: "Paneling",
    tag: "Paneling",
    child: {
      key: "panelingChild",
      title: "Subcategory (Optional)",
      selectionType: "single",
      filterType: "match_all",
      options: [
        {
          label: "Cork",
          tag: "Cork",
        },
        {
          label: "Laminate",
          tag: "Laminate",
        },
        {
          label: "Plastic & Synthetics",
          tag: "Plastic & Synthetics",
        },
        {
          label: "Wood & Wood Alternatives",
          tag: "Wood & Wood Alternatives",
        },
      ],
    },
  },
  {
    label: "Hardware",
    tag: "Hardware",
  },
  {
    label: "Faucets",
    tag: "Faucets",
  },
  {
    label: "Cabinets",
    tag: "Cabinets",
  },
  {
    label: "Decking",
    tag: "Floor & Deck",
  },
  {
    label: "Area Rugs",
    tag: "Area Rugs",
  },
  {
    label: "Window Shade",
    tag: "Window Shade",
  },
  {
    label: "Ceiling",
    tag: "Ceiling",
  },
];

export default function ECommerce() {
  const baseUrl = import.meta.env.VITE_API_BASE_URL;
  const datasetId = import.meta.env.VITE_DATASET_ID;
  const tutorialUrl = import.meta.env.VITE_TUTORIAL_URL;
  const apiKey = import.meta.env.VITE_API_KEY;

  return (
    <>
      <div className="min-w-screen max-w-[96rem] mx-auto min-h-screen relative p-2">
        <div className="w-full flex justify-between my-2 py-4 px-2 border-b">
          <p className="text-3xl font-bold">Design Muse</p>
          <a
            href={tutorialUrl}
            target="_blank"
            rel="noopener noreferrer"
            data-slot="button"
            className="cursor-pointer justify-center whitespace-nowrap text-sm transition-all disabled:pointer-events-none disabled:opacity-50 [&amp;_svg]:pointer-events-none [&amp;_svg:not([class*='size-'])]:size-4 shrink-0 [&amp;_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50 h-8 rounded-xl px-3 has-[>svg]:px-2.5 flex items-center gap-1 hover:bg-neutral-100 font-semibold"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.87a.5.5 0 0 0-.752-.432L16 10.5"></path>
              <rect x="2" y="6" width="14" height="12" rx="2"></rect>
            </svg>
            Watch Tutorial
          </a>
        </div>
        <div className="w-full">
          <TrieveModalSearch
            displayModal={false}
            datasetId={datasetId}
            apiKey={apiKey}
            baseUrl={baseUrl}
            inline={true}
            defaultSearchMode="chat"
            allowSwitchingModes={false}
            brandFontFamily="Karla"
            brandColor="#000"
            type="ecommerce"
            searchOptions={{
              search_type: "hybrid",
              page_size: 20,
            }}
            inlineCarousel={true}
            searchPageProps={{
              filterSidebarProps: {
                sections: [
                  {
                    key: "categories",
                    title: "Category",
                    selectionType: "single",
                    filterType: "match_all",
                    options,
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
                    placeholder:
                      "Click or drag to upload (HEIC, PNG, WEBP, JPG - Max 5MB)",
                  },
                  {
                    title: "What's in the image",
                    description:
                      "Understand how the AI sees your image and make adjustments to get the best results.",
                    type: "text",
                    filterSidebarSectionKey: "description",
                    prompt:
                      "Write a single short sentence (maximum 10 words) describing in high detail the way you see the space in the image in terms of color, luminance, and style. Make a specific callout to something unique so a reader knows you actually saw the image. \n\n",
                    inferenceInputLabel:
                      "How the AI understands your image (editable)",
                    inputLabel: "Describe the goal of the change",
                    placeholder:
                      "I want to make the space more modern and bright.",
                  },
                  {
                    title: "Filter Selection",
                    description:
                      "Select the material(s) you want to change and are interested in getting recommendations for.",
                    type: "tags",
                    filterSidebarSectionKey: "categories",
                  },
                  {
                    title: "View Recommended Materials",
                    description:
                      "Our AI will recommend materials based on your image and which materials you are replacing.",
                    type: "search_modal",
                    prompt:
                      "Taking the space and other details into consideration, write 1 sentence describing the ideal replacements in terms of color, luminance, and style of ONLY the following materials:\n\n",
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
