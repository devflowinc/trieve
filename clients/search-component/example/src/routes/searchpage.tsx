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
    label: "Wallpaper",
    tag: "Wallpaper",
    child: {
      key: "wallpaperChild",
      title: "Wallpaper Options",
      selectionType: "multiple",
      filterType: "match_any",
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
    label: "Countertops & Stone",
    tag: "Countertops & Stone",
    child: {
      key: "countertopsChild",
      title: "Countertops & Stone Options",
      selectionType: "multiple",
      filterType: "match_any",
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
  },
  {
    label: "Wood Flooring",
    tag: "Wood Flooring",
  },
  {
    label: "LVT & Specialty Flooring",
    tag: "LVT & Specialty Flooring",
    child: {
      key: "lvtFlooringChild",
      title: "LVT & Specialty Flooring Options",
      selectionType: "multiple",
      filterType: "match_any",
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
          label: "Resilient",
          tag: "Resilient",
        },
      ],
    },
  },
  {
    label: "Paint",
    tag: "Paint",
  },
  {
    label: "Carpet & Carpet Tile",
    tag: "Carpet & Carpet Tile",
  },
  {
    label: "Fabric & Leather",
    tag: "Fabric & Leather",
    child: {
      key: "fabricLeatherChild",
      title: "Fabric & Leather Options",
      selectionType: "multiple",
      filterType: "match_any",
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
          tag: "Velvet",
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
      title: "Paneling Options",
      selectionType: "multiple",
      filterType: "match_any",
      options: [
        {
          label: "Decorative",
          tag: "Decorative",
        },
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
    child: {
      key: "fixtureTypeChild",
      title: "Faucets Type Options",
      selectionType: "multiple",
      filterType: "match_any",
      options: [
        {
          label: "Accessories",
          tag: "Accessories",
        },
        {
          label: "Bar Faucets",
          tag: "Bar Faucets",
        },
        {
          label: "Bath Faucets",
          tag: "Bath Faucets",
        },
        {
          label: "Claw Foot Tubs",
          tag: "Claw Foot Tubs",
        },
        {
          label: "Console Sink",
          tag: "Console Sink",
        },
        {
          label: "Drop In Tubs",
          tag: "Drop In Tubs",
        },
        {
          label: "Drop-In Sink",
          tag: "Drop-In Sink",
        },
        {
          label: "Freestanding Tubs",
          tag: "Freestanding Tubs",
        },
        {
          label: "Kitchen Faucets",
          tag: "Kitchen Faucets",
        },
        {
          label: "Pedestal Sink",
          tag: "Pedestal Sink",
        },
        {
          label: "Pot Fillers",
          tag: "Pot Fillers",
        },
        {
          label: "Semi-Recessed Sink",
          tag: "Semi-Recessed Sink",
        },
        {
          label: "Shower & Tub Trim",
          tag: "Shower & Tub Trim",
        },
        {
          label: "Shower Only Trim",
          tag: "Shower Only Trim",
        },
        {
          label: "Undermount Sink",
          tag: "Undermount Sink",
        },
        {
          label: "Vessel Sink",
          tag: "Vessel Sink",
        },
        {
          label: "Wall Hung Vanity Cabinet",
          tag: "Wall Hung Vanity Cabinet",
        },
        {
          label: "Wall-Mounted Sink",
          tag: "Wall-Mounted Sink",
        },
      ],
    },
  },
  {
    label: "Cabinets",
    tag: "Cabinets",
  },
  {
    label: "Decking",
    tag: "Decking",
  },
  {
    label: "Area Rugs",
    tag: "Area Rugs",
  },
  {
    label: "Blinds & Shades",
    tag: "Blinds & Shades",
  },
  {
    label: "Ceiling",
    tag: "Ceiling",
    child: {
      key: "ceilingChild",
      title: "Ceiling Options",
      selectionType: "multiple",
      filterType: "match_any",
      options: [
        {
          label: "Panel",
          tag: "Panel",
        },
        {
          label: "Tile",
          tag: "Tile",
        },
      ],
    },
  },
];

export default function ECommerce() {
  const baseUrl = import.meta.env.VITE_API_BASE_URL;
  const datasetId = import.meta.env.VITE_DATASET_ID;
  const apiKey = import.meta.env.VITE_API_KEY;

  return (
    <>
      <div className="min-w-screen min-h-screen relative">
        <div className="w-full">
          <TrieveModalSearch
            displayModal={false}
            datasetId={datasetId}
            apiKey={apiKey}
            baseUrl={baseUrl}
            inline={true}
            defaultSearchMode="chat"
            allowSwitchingModes={false}
            brandColor="#a33eb5"
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
                    filterType: "match_any",
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
                  },
                  {
                    title: "Filter Selection",
                    description:
                      "Select the material(s) you want to change and are interested in getting recommendations for.",
                    type: "tags",
                    filterSidebarSectionKey: "categories",
                    inputLabel: "Describe the goal of the change",
                    placeholder:
                      "I want to make the space more modern and bright.",
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
