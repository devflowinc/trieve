import "../../../dist/index.css";
import "../custom-css/searchpage.css";
import { TrieveModalSearch } from "../../../src/index";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/searchpage")({
  component: ECommerce,
});

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
            inlineCarousel={true}
            searchPageProps={{
              filterSidebarProps: {
                sections: [
                  {
                    key: "categories",
                    title: "Categories",
                    options: [
                      {
                        label: "Backsplash",
                        tag: "Backsplash",
                        description:
                          "Set this to true anytime the image seems to include backsplashes or a kitchen counter in general.",
                      },
                      {
                        label: "Flooring",
                        tag: "Flooring",
                        description:
                          "Set this to true anytime the image seems to include floors.",
                      },
                      {
                        label: "Countertops",
                        tag: "Countertops",
                        description:
                          "Set this to true anytime the image seems to include countertops or something that looks vaguely like a countertop.",
                      },
                      {
                        label: "Cabinets",
                        tag: "Cabinets",
                        description:
                          "Set this to true anytime the image seems to include cabinets.",
                      },
                      {
                        label: "Hardware",
                        tag: "Hardware",
                        description:
                          "Set this to true anytime the image seems to include or like it could include hardware like cabinet handles or knobs.",
                      },
                      {
                        label: "Sinks",
                        tag: "Sinks",
                        description:
                          "Set this to true anytime the image seems to include sinks.",
                      },
                      {
                        label: "Rugs",
                        tag: "Rugs",
                        description:
                          "Set this to true anytime the image seems to include or like it could include a rug.",
                      },
                      {
                        label: "Indoor Wall Covering",
                        tag: "Wallcovering",
                        description:
                          "Set this to true anytime the image seems to include indoor walls. If paint is present, set this to true as well. Literally any time you see a wall, set this to true.",
                      },
                      {
                        label: "Paint",
                        tag: "Paint",
                        description:
                          "Set this to true anytime the image seems to include walls. If wallcovering is present, set this to true as well. Literally any time you see a wall, set this to true.",
                      },
                      {
                        label: "Hardwood Flooring",
                        tag: "Engineered Hardwood",
                        description:
                          "Set this to true anytime the image seems to include or like it could include hardwood flooring.",
                      },
                      {
                        label: "Bathroom Flooring",
                        tag: "Bathroom Flooring",
                        description:
                          "Set this to true anytime the image seems to include or like it could include bathroom flooring.",
                      },
                      {
                        label: "Shower Wall",
                        tag: "Shower Wall",
                        description:
                          "Set this to true anytime the image seems to include or like it could include shower walls. Specifically shower. Bathrooms in general should have this set to true.",
                      },
                      {
                        label: "Fabric",
                        tag: "Fabric",
                        description:
                          "Set this to true anytime the image seems to include or like it could include fabric. Upholstery, curtains, etc.",
                      },
                      {
                        label: "Home Exterior Patterns",
                        tag: "Home Exterior",
                        description:
                          "Set this to true anytime the image seems to include or like it could include outdoor walls. Literally if you see an outdoor space, set this to true.",
                      },
                    ],
                    selectionType: "single",
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
                      "Write 1 sentence describing in high detail the way you see the space in the image in terms of color, luminance, and style. Make a specific callout to something unique so a reader knows you actually saw the image. \n\n",
                    inferenceInputLabel:
                      "How the AI understands your image (editable)",
                  },
                  {
                    title: "Category Selection",
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
