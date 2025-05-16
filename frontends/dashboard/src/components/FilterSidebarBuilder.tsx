import { createStore } from "solid-js/store";
import { createSignal, For, Show } from "solid-js";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { FiPlus, FiTrash } from "solid-icons/fi";
import { Select, Tooltip } from "shared/ui";
import { usePublicPage } from "../hooks/usePublicPageSettings";
import {
  FilterSidebarSection,
  TagProp,
} from "../../../../clients/ts-sdk/dist/types.gen";

const FilterSidebarBuilder = () => {
  const { extraParams, setExtraParams } = usePublicPage();

  // Initialize sections from existing data or with an empty array
  const initialSections =
    extraParams.searchPageProps?.filterSidebarProps?.sections || [];
  const [sections, setSections] =
    createStore<FilterSidebarSection[]>(initialSections);

  // Track which section is currently selected for editing
  const [selectedSectionIndex, setSelectedSectionIndex] =
    createSignal<number>(0);

  // Filter type options
  const filterTypeOptions = [
    { label: "Match Any", value: "match_any" },
    { label: "Match All", value: "match_all" },
    { label: "Range", value: "range" },
  ];

  // Selection type options
  const selectionTypeOptions = [
    { label: "Single", value: "single" },
    { label: "Multiple", value: "multiple" },
  ];

  // Function to add a new section
  const addSection = () => {
    const newSection: FilterSidebarSection = {
      key: `section-${sections.length + 1}`,
      filterKey: `filter-${sections.length + 1}`,
      title: `Section ${sections.length + 1}`,
      selectionType: "single",
      filterType: "match_any",
      options: [],
    };

    setSections([...sections, newSection]);
    setSelectedSectionIndex(sections.length - 1);
  };

  // Function to delete a section
  const deleteSection = (index: number) => {
    setSections([...sections.slice(0, index), ...sections.slice(index + 1)]);
    setSelectedSectionIndex(sections.length - 1);
  };

  // Function to add option to a section
  const addOption = (sectionIndex: number) => {
    const newOption: TagProp = {
      tag: null,
      label: null,
      range: null,
    };

    setSections(sectionIndex, "options", [
      ...sections[sectionIndex].options,
      newOption,
    ]);
  };

  // Function to delete an option
  const deleteOption = (sectionIndex: number, optionIndex: number) => {
    setSections(sectionIndex, "options", [
      ...sections[sectionIndex].options.slice(0, optionIndex),
      ...sections[sectionIndex].options.slice(optionIndex + 1),
    ]);
  };

  // Function to update state in extraParams when sections change
  const updateSerpPageOptions = () => {
    setExtraParams("searchPageProps", {
      ...extraParams.searchPageProps,
      filterSidebarProps: {
        sections: sections,
      },
    });
  };

  // Section Editor Component
  const SectionEditor = (props: {
    section: FilterSidebarSection;
    index: number;
  }) => {
    return (
      <div class="relative border border-neutral-200 p-4">
        <button
          onClick={() => deleteSection(props.index)}
          class="absolute right-2 top-2 flex items-center gap-2 rounded border border-neutral-200 bg-neutral-100 p-1 text-sm font-medium text-neutral-500 hover:bg-neutral-200"
        >
          <FiTrash />
          Delete Section
        </button>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block">Section Key (ID)</label>
            <input
              placeholder="section-1"
              value={props.section.key}
              onInput={(e) => {
                setSections(props.index, "key", e.currentTarget.value);
                updateSerpPageOptions();
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>

          <div>
            <label class="block">Filter Key</label>
            <input
              placeholder="filter-1"
              value={props.section.filterKey}
              onInput={(e) => {
                setSections(props.index, "filterKey", e.currentTarget.value);
                updateSerpPageOptions();
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>

          <div>
            <label class="block">Section Title</label>
            <input
              placeholder="Categories"
              value={props.section.title}
              onInput={(e) => {
                setSections(props.index, "title", e.currentTarget.value);
                updateSerpPageOptions();
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>

          <div>
            <div class="flex items-center gap-1">
              <label class="block">Filter Type</label>
              <Tooltip
                tooltipText="Type of filter: Match Any or Match All for categorical filters, Range for numeric ranges"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <Select
              display={(option) => option?.label || "Match Any"}
              onSelected={(option) => {
                setSections(
                  props.index,
                  "filterType",
                  option?.value || "match_any",
                );
                updateSerpPageOptions();
              }}
              class="bg-white py-1"
              selected={
                filterTypeOptions.find(
                  (option) => option.value === props.section.filterType,
                ) || filterTypeOptions[0]
              }
              options={filterTypeOptions}
            />
          </div>
          <Show when={props.section.filterType !== "range"}>
            <div>
              <div class="flex items-center gap-1">
                <label class="block">Selection Type</label>
                <Tooltip
                  tooltipText="Single allows only one selection, multiple allows many"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <Select
                display={(option) => option?.label || "Single"}
                onSelected={(option) => {
                  setSections(
                    props.index,
                    "selectionType",
                    option?.value || "single",
                  );
                  updateSerpPageOptions();
                }}
                class="bg-white py-1"
                selected={
                  selectionTypeOptions.find(
                    (option) => option.value === props.section.selectionType,
                  ) || selectionTypeOptions[0]
                }
                options={selectionTypeOptions}
              />
            </div>
          </Show>
        </div>

        <div class="mt-4">
          <div class="flex items-center justify-between">
            <h4 class="font-medium">Filter Options</h4>
            <button
              onClick={() => addOption(props.index)}
              class="flex items-center gap-1 rounded border border-neutral-300 bg-white px-2 py-1 text-sm hover:bg-neutral-100"
            >
              <FiPlus size={14} />
              Add Option
            </button>
          </div>

          <div class="mt-2 space-y-3">
            <For each={props.section.options}>
              {(option, optionIndex) => (
                <div class="relative rounded border border-neutral-100 bg-neutral-50 p-3">
                  <button
                    onClick={() => deleteOption(props.index, optionIndex())}
                    class="absolute right-2 top-2 rounded border border-neutral-200 bg-white p-1 text-xs text-neutral-500 hover:bg-neutral-100"
                  >
                    <FiTrash size={12} />
                  </button>

                  <div class="grid grid-cols-2 gap-4">
                    <Show
                      when={
                        props.section.filterType === "match_any" ||
                        props.section.filterType === "match_all"
                      }
                    >
                      <div>
                        <label class="block text-sm">Tag Value</label>
                        <input
                          placeholder="tag-value"
                          value={option.tag || ""}
                          onInput={(e) => {
                            setSections(
                              props.index,
                              "options",
                              optionIndex(),
                              "tag",
                              e.currentTarget.value,
                            );
                            updateSerpPageOptions();
                          }}
                          class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                        />
                      </div>
                    </Show>
                    <Show
                      when={
                        props.section.filterType === "match_all" ||
                        props.section.filterType === "match_any"
                      }
                    >
                      <div>
                        <label class="block text-sm">Display Label</label>
                        <input
                          placeholder="Display Label"
                          value={option.label || ""}
                          onInput={(e) => {
                            setSections(
                              props.index,
                              "options",
                              optionIndex(),
                              "label",
                              e.currentTarget.value,
                            );
                            updateSerpPageOptions();
                          }}
                          class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                        />
                      </div>
                    </Show>

                    <Show when={props.section.filterType === "range"}>
                      <div>
                        <label class="block text-sm">Min Value</label>
                        <input
                          type="number"
                          placeholder="0"
                          value={option.range?.min || 0}
                          onInput={(e) => {
                            setSections(
                              props.index,
                              "options",
                              optionIndex(),
                              "range",
                              {
                                ...option.range,
                                min: parseFloat(e.currentTarget.value),
                              },
                            );
                            updateSerpPageOptions();
                          }}
                          class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                        />
                      </div>

                      <div>
                        <label class="block text-sm">Max Value</label>
                        <input
                          type="number"
                          placeholder="100"
                          value={option.range?.max || 100}
                          onInput={(e) => {
                            setSections(
                              props.index,
                              "options",
                              optionIndex(),
                              "range",
                              {
                                ...option.range,
                                max: parseFloat(e.currentTarget.value),
                              },
                            );
                            updateSerpPageOptions();
                          }}
                          class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                        />
                      </div>
                    </Show>
                    <div>
                      <label class="block text-sm">
                        Description (This will be used to help the AI filter the
                        options)
                      </label>
                      <textarea
                        placeholder="Description"
                        value={option.description || ""}
                        onInput={(e) => {
                          setSections(
                            props.index,
                            "options",
                            optionIndex(),
                            "description",
                            e.currentTarget.value,
                          );
                          updateSerpPageOptions();
                        }}
                        class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                      />
                    </div>
                  </div>
                </div>
              )}
            </For>

            <Show when={props.section.options.length === 0}>
              <div class="rounded border border-dashed border-neutral-300 p-4 text-center text-sm text-neutral-500">
                No options added yet. Click "Add Option" to create filter
                options.
              </div>
            </Show>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div class="mt-4">
      <div class="flex items-center justify-between pb-2">
        <div class="flex items-center gap-1">
          <h3 class="font-medium">Filter Sidebar Sections</h3>
          <Tooltip
            tooltipText="Create filter sections that will appear in the sidebar of the SERP page"
            body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
          />
        </div>
      </div>

      <div class="flex items-end gap-2 overflow-y-auto pt-2">
        <For each={sections}>
          {(section, index) => (
            <div class="flex flex-row gap-2">
              <button
                onClick={() => setSelectedSectionIndex(index())}
                classList={{
                  "bg-neutral-200/70 border-neutral-200 border hover:bg-neutral-200 p-2 px-4 rounded-t-md":
                    true,
                  "!bg-magenta-100/50 border-transparent hover:bg-magenta-100/80 text-magenta-900":
                    index() === selectedSectionIndex(),
                }}
              >
                {section.title || `Section ${index() + 1}`}
              </button>
            </div>
          )}
        </For>

        <button
          onClick={addSection}
          classList={{
            "ml-4 rounded flex items-center gap-2 border border-neutral-300 hover:bg-neutral-200 py-1 bg-neutral-100 p-2":
              true,
            "border-b-transparent": selectedSectionIndex() !== null,
          }}
        >
          <FiPlus />
          Add Section
        </button>
      </div>

      <Show
        when={
          selectedSectionIndex() !== null &&
          sections[selectedSectionIndex() ?? 0]
        }
      >
        <SectionEditor
          index={selectedSectionIndex() ?? 0}
          section={sections[selectedSectionIndex() ?? 0]}
        />
      </Show>

      <Show when={sections.length === 0}>
        <div class="mt-2 rounded border border-dashed border-neutral-300 p-6 text-center text-neutral-500">
          No filter sections added yet. Click "Add Section" to create a filter
          section.
        </div>
      </Show>
    </div>
  );
};

export default FilterSidebarBuilder;
