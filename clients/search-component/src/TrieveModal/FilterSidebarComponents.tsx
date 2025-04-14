import React, { useEffect, useMemo } from "react";
import { useState } from "react";
import {
  CheckIcon,
  ChevronDownIcon,
  ChevronUpicon,
  LoadingIcon,
  PhotoIcon,
  XIcon,
} from "./icons";
import {
  InferenceFiltersFormProps,
  useModalState,
} from "../utils/hooks/modal-context";
import { toBase64 } from "./Search/UploadImage";
import { getPresignedUrl, uploadFile } from "../utils/trieve";
import { ModalContainer } from "./ModalContainer";
import { useChatState } from "../utils/hooks/chat-context";
import convert from "heic-convert/browser";

export const ActiveFilterPills = () => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();

  const activeFilters: {
    sectionKey: string;
    tags: string[];
  }[] = useMemo(() => {
    const filters = Object.entries(selectedSidebarFilters).map(
      ([sectionKey, tags]) => ({
        sectionKey,
        tags,
      }),
    );
    return filters;
  }, [selectedSidebarFilters]);

  const numberOfSelectedFilters = useMemo(() => {
    let count = 0;
    for (const { sectionKey } of activeFilters) {
      if (sectionKey in selectedSidebarFilters) {
        count += selectedSidebarFilters[sectionKey].length;
      }
    }
    return count;
  }, [selectedSidebarFilters]);

  return (
    <div
      className="trieve-active-filter-pills-container"
      data-number-selected-filters={numberOfSelectedFilters}
    >
      <div className="trieve-all-active-filters">
        {activeFilters.map(({ sectionKey, tags }) =>
          tags.map((tag) => (
            <button
              className="trieve-active-filter-pill"
              key={tag}
              onClick={() => {
                setSelectedSidebarFilters((prev) => ({
                  ...prev,
                  [sectionKey]: prev[sectionKey].filter((t) => t !== tag),
                }));
              }}
            >
              <span>{tag}</span>
              <i className="trieve-active-filter-pill-remove-icon">
                <XIcon />
              </i>
            </button>
          )),
        )}
      </div>
      <button
        className="trieve-clear-filters-button"
        data-number-selected-filters={numberOfSelectedFilters}
        onClick={() => {
          setSelectedSidebarFilters({});
        }}
      >
        Clear all
      </button>
    </div>
  );
};

export interface AccordionProps {
  sectionKey: string;
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  onToggle?: () => void;
}

export const Accordion = ({
  sectionKey,
  title,
  children,
  defaultOpen = false,
  onToggle,
}: AccordionProps) => {
  const { selectedSidebarFilters } = useModalState();
  const [open, setOpen] = useState(defaultOpen);

  const numberOfSelectedFilters = useMemo(() => {
    if (sectionKey in selectedSidebarFilters) {
      return selectedSidebarFilters[sectionKey].length;
    }
    return 0;
  }, [sectionKey, selectedSidebarFilters]);

  return (
    <div
      className="trieve-accordion-container"
      data-open={open ? "true" : "false"}
    >
      <div
        className="trieve-accordion-header"
        data-open={open ? "true" : "false"}
        onClick={() => {
          setOpen(!open);
          if (onToggle) {
            onToggle();
          }
        }}
      >
        <h3 className="trieve-accordion-title">{title}</h3>
        <div className="trieve-accordion-details">
          <span
            className="trieve-accordion-number"
            data-value={numberOfSelectedFilters}
          >
            {numberOfSelectedFilters}
          </span>
          <div className="trieve-accordion-icon-container">
            {open ? <ChevronUpicon /> : <ChevronDownIcon />}
          </div>
        </div>
      </div>
      <div
        className="trieve-accordion-content-container"
        data-open={open ? "true" : "false"}
      >
        <div className="trieve-accordion-content">{children}</div>
        <hr />
      </div>
    </div>
  );
};

export interface FilterButtonProps {
  sectionKey: string;
  filterKey: string;
  label: string;
  type: "single" | "multiple";
  description?: string;
  onClick?: () => void;
  isChild?: boolean;
}

export const FilterButton = ({
  sectionKey,
  filterKey,
  label,
  description,
  type,
  onClick,
  isChild,
}: FilterButtonProps) => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();

  const active = useMemo(() => {
    if (sectionKey in selectedSidebarFilters) {
      const selectedFilters = selectedSidebarFilters[sectionKey];
      return selectedFilters.includes(filterKey);
    }
    return false;
  }, [sectionKey, filterKey, selectedSidebarFilters]);

  const handleClick = () => {
    if (type === "single") {
      if (active) {
        setSelectedSidebarFilters((prev) => {
          if (isChild) {
            return {
              ...prev,
              [sectionKey]: prev[sectionKey].filter(
                (item) => item !== filterKey,
              ),
            };
          }

          return {
            ...prev,
            [sectionKey]: [],
          };
        });
      } else {
        setSelectedSidebarFilters((prev) => {
          if (isChild) {
            const currentFilters = prev[sectionKey] || [];
            const newFilters = [];
            if (currentFilters.length > 0) {
              newFilters.push(currentFilters[0]);
            }
            newFilters.push(filterKey);

            return {
              ...prev,
              [sectionKey]: newFilters,
            };
          }

          return {
            ...selectedSidebarFilters,
            [sectionKey]: [filterKey],
          };
        });
      }
    } else {
      if (active) {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: selectedSidebarFilters[sectionKey].filter(
            (item) => item !== filterKey,
          ),
        });
      } else {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: [
            ...(selectedSidebarFilters[sectionKey] || []),
            filterKey,
          ],
        });
      }
    }
    if (onClick) onClick();
  };

  return (
    <button className="trieve-filter-button-container" onClick={handleClick}>
      <div
        className={`trieve-${type}-button`}
        data-active={active ? "true" : "false"}
      >
        <div className="trieve-circle" />
        <i className="trieve-checkbox-icon">
          <CheckIcon />
        </i>
      </div>
      <label className="trieve-filter-button-label" title={description}>
        {label}
      </label>
    </button>
  );
};

export interface SearchQueryState {
  query: string;
  loading: boolean;
}

export interface TextFieldState {
  inferenceValue: string;
  inputValue?: string;
  loading: boolean;
}

export interface InferenceFilterFormStep {
  title: string;
  description: string;
  type: "image" | "tags" | "search_modal" | "text";
  placeholder?: string;
  filterSidebarSectionKey?: string;
  prompt?: string;
  inferenceInputLabel?: string;
  inputLabel?: string;
}

export const InferenceFiltersForm = ({ steps }: InferenceFiltersFormProps) => {
  const {
    trieveSDK,
    props,
    setSelectedSidebarFilters,
    selectedSidebarFilters,
  } = useModalState();
  const { fingerprint } = useModalState();
  const { askQuestion, clearConversation, stopGeneratingMessage } =
    useChatState();
  const [images, setImages] = useState<Record<string, File>>({});
  const [imageUrls, setImageUrls] = useState<Record<string, string>>({});
  const [textFields, setTextFields] = useState<Record<string, TextFieldState>>(
    {},
  );
  const [loadingStates, setLoadingStates] = useState<Record<string, string>>(
    {},
  );
  const [filterOptions, setFilterOptions] = useState<Record<string, string[]>>(
    {},
  );
  const [completedSteps, setCompletedSteps] = useState<Record<string, boolean>>(
    {},
  );

  useEffect(() => {
    const toolCallAbortController = new AbortController();

    for (let i = 1; i < steps.length; i++) {
      if (completedSteps[steps[i].title]) {
        continue;
      }

      const correspondingFilter =
        props.searchPageProps?.filterSidebarProps?.sections.find(
          (section) => section.key === steps[i].filterSidebarSectionKey,
        );
      if (!correspondingFilter?.options) {
        continue;
      }

      const prevStep = steps[i - 1];
      if (!completedSteps[prevStep.title]) {
        continue;
      }

      (async () => {
        const match_all_tags: string[] = [];
        if (match_all_tags.length === 0) {
          match_all_tags.push(...correspondingFilter.options.map((t) => t.tag));
        }

        setFilterOptions((prev) => {
          const newFilterOptions = {
            ...prev,
            [steps[i].filterSidebarSectionKey ?? ""]: match_all_tags,
          };

          return newFilterOptions;
        });

        setLoadingStates((prev) => ({
          ...prev,
          [steps[i].title]: "idle",
        }));
      })();
    }

    return () => {
      toolCallAbortController.abort();
    };
  }, [completedSteps]);

  useEffect(() => {
    const firstMessageInferenceAbortController = new AbortController();
    for (let i = 1; i < steps.length; i++) {
      if (completedSteps[steps[i].title]) {
        continue;
      }

      if (steps[i].type === "search_modal") {
        const prevFilter = steps[i - 1].filterSidebarSectionKey;
        const selectedTags = selectedSidebarFilters[prevFilter ?? ""];
        if (!completedSteps[steps[i - 1].title]) {
          continue;
        }

        (async () => {
          setLoadingStates((prev) => ({
            ...prev,
            [steps[i].title]: "Figuring out what will look good...",
          }));
          stopGeneratingMessage();
          clearConversation();

          const prevStep = i > 0 ? steps[i - 1] : null;
          const prevInferenceText = prevStep
            ? (textFields[prevStep.title]?.inferenceValue ?? "")
            : "";
          const prevInputText = prevStep
            ? (textFields[prevStep.title]?.inputValue ?? "")
            : "";
          let promptDescription = `${steps[i].prompt ?? ""} ${selectedTags.join(", ")}`;
          if (prevInferenceText) {
            promptDescription += `\n\n[Context for the existing space]:\n${prevInferenceText}`;
          }
          if (prevInputText) {
            promptDescription += `\n\n[User's goal for the space (take more into account than anything else)]:\n${prevInputText}`;
          }

          const replacementMaterialDescriptionReader =
            await trieveSDK.ragOnChunkReader(
              {
                chunk_ids: [],
                image_urls: Object.values(imageUrls).filter((url) => url),
                prev_messages: [
                  {
                    content: promptDescription,
                    role: "user",
                  },
                ],
                prompt: "",
                stream_response: true,
                user_id: fingerprint.toString(),
              },
              firstMessageInferenceAbortController.signal,
            );
          setLoadingStates((prev) => ({
            ...prev,
            [steps[i].title]: "Generating search query...",
          }));

          let done = false;
          let textInStream = "";
          while (!done) {
            const { value, done: doneReading } =
              await replacementMaterialDescriptionReader.read();
            if (doneReading) {
              done = doneReading;
              setLoadingStates((prev) => ({
                ...prev,
                [steps[i].title]: "idle",
              }));
              if (prevInferenceText) {
                textInStream += `\n\n[Context on my space]:\n${prevInferenceText}`;
              }
              if (prevInputText) {
                textInStream += `\n\n[My goal for the space]:\n${prevInputText}`;
              }

              askQuestion(textInStream, undefined, false);

              setCompletedSteps((prev) => ({
                ...prev,
                [steps[i].title]: true,
              }));
            } else if (value) {
              const decoder = new TextDecoder();
              const newText = decoder.decode(value);
              textInStream += newText;
            }
          }
        })();
      }
    }

    return () => {
      firstMessageInferenceAbortController.abort();
    };
  }, [completedSteps]);

  useEffect(() => {
    const textInferenceAbortController = new AbortController();
    for (let i = 1; i < steps.length; i++) {
      if (steps[i].type === "text") {
        const prevStep = steps[i - 1];
        const image_url = imageUrls[prevStep.title];
        if (!image_url) {
          continue;
        }

        (async () => {
          setLoadingStates((prev) => ({
            ...prev,
            [steps[i].title]: "Understanding your space...",
          }));

          const replacementMaterialDescriptionReader =
            await trieveSDK.ragOnChunkReader(
              {
                chunk_ids: [],
                image_urls: Object.values(imageUrls).filter((url) => url),
                prev_messages: [
                  {
                    content: `${steps[i].prompt ?? ""}`,
                    role: "user",
                  },
                ],
                prompt: "",
                stream_response: true,
                user_id: fingerprint.toString(),
              },
              textInferenceAbortController.signal,
            );

          setLoadingStates((prev) => ({
            ...prev,
            [steps[i].title]: "Generating style analysis...",
          }));

          let done = false;
          let textInStream = "";
          while (!done) {
            const { value, done: doneReading } =
              await replacementMaterialDescriptionReader.read();
            if (doneReading) {
              done = doneReading;
              setLoadingStates((prev) => ({
                ...prev,
                [steps[i].title]: "idle",
              }));
              setTextFields((prev) => ({
                ...prev,
                [steps[i].title]: {
                  inferenceValue: textInStream,
                  loading: false,
                },
              }));
            } else if (value) {
              const decoder = new TextDecoder();
              const newText = decoder.decode(value);
              textInStream += newText;
              setTextFields((prev) => ({
                ...prev,
                [steps[i].title]: {
                  inferenceValue: textInStream,
                  loading: false,
                },
              }));
            }
          }
        })();
      }
    }

    return () => {
      textInferenceAbortController.abort();
    };
  }, [imageUrls]);

  return (
    <div className="trieve-inference-filters-form">
      {steps.map((step, index) => (
        <div
          className="trieve-inference-filters-step-container"
          key={index}
          data-prev-complete={
            index == 0 || completedSteps[steps[index - 1].title]
              ? "true"
              : "false"
          }
        >
          <div className="trieve-inference-filters-step-header">
            <div
              className="trieve-inference-filters-step-number"
              data-completed={images[step.title] ? "true" : "false"}
            >
              <span>{index + 1}</span>
            </div>
            <h2 className="trieve-inference-filters-step-title">
              {step.title}
            </h2>
          </div>
          <p className="trieve-inference-filters-step-description">
            {step.description}
          </p>
          <div
            className="trieve-inference-filters-step-input-container"
            data-loading-state={loadingStates[step.title] ?? "idle"}
          >
            <div
              className="trieve-image-input-container"
              data-input-field-type={step.type}
              data-image-selected={images[step.title] ? "true" : "false"}
              onDragOver={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onDrop={async (e) => {
                e.preventDefault();
                e.stopPropagation();
                stopGeneratingMessage();
                clearConversation();
                setCompletedSteps((prev) => {
                  const newCompletedSteps = { ...prev };
                  for (let j = index; j < steps.length; j++) {
                    newCompletedSteps[steps[j].title] = false;
                  }
                  return newCompletedSteps;
                });
                setSelectedSidebarFilters((prev) => {
                  for (let j = index; j < steps.length; j++) {
                    prev[steps[j].filterSidebarSectionKey ?? ""] = [];
                  }
                  return prev;
                });
                setTextFields((prev) => {
                  const newTextFields = { ...prev };
                  for (let j = index; j < steps.length; j++) {
                    newTextFields[steps[j].title] = {
                      inferenceValue: "",
                      inputValue: "",
                      loading: false,
                    };
                  }
                  return newTextFields;
                });
                const files = e.dataTransfer.files;
                let processedFile =
                  (files?.length ?? 1) > 0 ? files?.[0] : null;
                if (!processedFile) {
                  return;
                }

                if (
                  processedFile.type === "image/heic" ||
                  processedFile.name.toLowerCase().endsWith(".heic")
                ) {
                  try {
                    const buffer = await processedFile.arrayBuffer();
                    const convertedFile = await convert({
                      buffer: new Uint8Array(buffer) as unknown as ArrayBuffer,
                      format: "PNG",
                    });
                    processedFile = new File(
                      [convertedFile],
                      processedFile.name.replace(/\.heic$/i, ".png"),
                      {
                        type: "image/png",
                        lastModified: Date.now(),
                      },
                    );
                  } catch (err) {
                    console.error("HEIC conversion failed:", err);
                    return;
                  }
                }

                setImages((prev) => ({
                  ...prev,
                  [step.title]: processedFile,
                }));
                setLoadingStates((prev) => ({
                  ...prev,
                  [step.title]: "Uploading image...",
                }));
                toBase64(processedFile).then((data) => {
                  const base64File = data
                    .split(",")[1]
                    .replace(/\+/g, "-")
                    .replace(/\//g, "_")
                    .replace(/=+$/, "");
                  uploadFile(trieveSDK, processedFile.name, base64File).then(
                    (fileId) => {
                      getPresignedUrl(trieveSDK, fileId).then((imageUrl) => {
                        setImageUrls((prev) => ({
                          ...prev,
                          [step.title]: imageUrl,
                        }));

                        setLoadingStates((prev) => ({
                          ...prev,
                          [step.title]: "idle",
                        }));

                        setCompletedSteps((prev) => ({
                          ...prev,
                          [step.title]: true,
                        }));
                      });
                    },
                  );
                });
              }}
              onClick={() => {
                const input = document.createElement("input");
                input.type = "file";
                input.accept = "image/*, .heic, .HEIC";
                input.multiple = false;
                input.onchange = async (e) => {
                  stopGeneratingMessage();
                  clearConversation();
                  setCompletedSteps((prev) => {
                    const newCompletedSteps = { ...prev };
                    for (let j = index; j < steps.length; j++) {
                      newCompletedSteps[steps[j].title] = false;
                    }
                    return newCompletedSteps;
                  });
                  setSelectedSidebarFilters((prev) => {
                    for (let j = index; j < steps.length; j++) {
                      prev[steps[j].filterSidebarSectionKey ?? ""] = [];
                    }
                    return prev;
                  });
                  setTextFields((prev) => {
                    const newTextFields = { ...prev };
                    for (let j = index; j < steps.length; j++) {
                      newTextFields[steps[j].title] = {
                        inferenceValue: "",
                        inputValue: "",
                        loading: false,
                      };
                    }
                    return newTextFields;
                  });
                  const files = (e.target as HTMLInputElement).files;
                  let processedFile =
                    (files?.length ?? 1) > 0 ? files?.[0] : null;
                  if (!processedFile) {
                    return;
                  }

                  if (
                    processedFile.type === "image/heic" ||
                    processedFile.name.toLowerCase().endsWith(".heic")
                  ) {
                    try {
                      const buffer = await processedFile.arrayBuffer();
                      const convertedFile = await convert({
                        buffer: new Uint8Array(
                          buffer,
                        ) as unknown as ArrayBuffer,
                        format: "PNG",
                      });
                      processedFile = new File(
                        [convertedFile],
                        processedFile.name.replace(/\.heic$/i, ".png"),
                        {
                          type: "image/png",
                          lastModified: Date.now(),
                        },
                      );
                    } catch (err) {
                      console.error("HEIC conversion failed:", err);
                      return;
                    }
                  }

                  setImages((prev) => ({
                    ...prev,
                    [step.title]: processedFile,
                  }));
                  setLoadingStates((prev) => ({
                    ...prev,
                    [step.title]: "Uploading image...",
                  }));
                  toBase64(processedFile).then((data) => {
                    const base64File = data
                      .split(",")[1]
                      .replace(/\+/g, "-")
                      .replace(/\//g, "_")
                      .replace(/=+$/, "");
                    uploadFile(trieveSDK, processedFile.name, base64File).then(
                      (fileId) => {
                        getPresignedUrl(trieveSDK, fileId).then((imageUrl) => {
                          setImageUrls((prev) => ({
                            ...prev,
                            [step.title]: imageUrl,
                          }));

                          setLoadingStates((prev) => ({
                            ...prev,
                            [step.title]: "idle",
                          }));

                          setCompletedSteps((prev) => ({
                            ...prev,
                            [step.title]: true,
                          }));
                        });
                      },
                    );
                  });
                };

                input.click();
              }}
            >
              <i
                className="trieve-image-input-icon"
                data-image-selected={images[step.title] ? "true" : "false"}
              >
                <PhotoIcon />
              </i>
              <img
                className="trieve-image-input-preview"
                src={
                  images[step.title]
                    ? URL.createObjectURL(images[step.title])
                    : ""
                }
                alt=""
                data-image-selected={images[step.title] ? "true" : "false"}
              />
              <p className="trieve-image-input-placeholder">
                {step.placeholder}
              </p>
            </div>

            <div
              className="trieve-text-step-container"
              data-input-field-type={step.type}
              data-prev-complete={
                index == 0 || completedSteps[steps[index - 1].title]
                  ? "true"
                  : "false"
              }
              data-completed={completedSteps[step.title] ? "true" : "false"}
            >
              <div className="trieve-text-input-container">
                <label
                  htmlFor="ai-understanding-input"
                  className="trieve-text-input-label"
                >
                  {step.inferenceInputLabel ?? "AI Understanding"}
                </label>
                <div className="trieve-text-input-textarea-container">
                  <textarea
                    name="ai-understanding-input"
                    rows={4}
                    className="trieve-text-input-textarea"
                    disabled={completedSteps[step.title]}
                    value={textFields[step.title]?.inferenceValue}
                    onChange={(e) => {
                      setTextFields((prev) => ({
                        ...prev,
                        [step.title]: {
                          ...prev[step.title],
                          inferenceValue: e.target.value,
                        },
                      }));
                    }}
                  />
                </div>
              </div>

              {step.inputLabel && (
                <div className="trieve-text-input-container">
                  <label
                    htmlFor="text-input"
                    className="trieve-text-input-label"
                  >
                    {step.inputLabel}
                  </label>
                  <div className="trieve-text-input-textarea-container">
                    <textarea
                      name="text-input"
                      rows={4}
                      className="trieve-text-input-textarea"
                      disabled={completedSteps[step.title]}
                      placeholder={step.placeholder}
                      value={textFields[step.title]?.inputValue}
                      onChange={(e) => {
                        setTextFields((prev) => ({
                          ...prev,
                          [step.title]: {
                            ...prev[step.title],
                            inputValue: e.target.value,
                          },
                        }));
                      }}
                    />
                  </div>
                </div>
              )}

              <div className="trieve-text-button-container">
                <button
                  type="button"
                  className="trieve-text-input-button"
                  onClick={() => {
                    setCompletedSteps((prev) => ({
                      ...prev,
                      [step.title]: true,
                    }));
                  }}
                >
                  Next
                  <div>
                    <i className="fa-solid fa-arrow-right"></i>
                  </div>
                </button>
              </div>
            </div>

            <div
              className="trieve-inference-filters-step-tags-container"
              data-input-field-type={step.type}
              data-prev-complete={
                index == 0 || completedSteps[steps[index - 1].title]
                  ? "true"
                  : "false"
              }
              data-completed={completedSteps[step.title] ? "true" : "false"}
            >
              <div
                className="trieve-inference-filters-step-tags"
                data-prev-complete={
                  index == 0 || completedSteps[steps[index - 1].title]
                    ? "true"
                    : "false"
                }
              >
                <p className="trieve-inference-filters-step-tags-section-title">
                  {props.searchPageProps?.filterSidebarProps?.sections.find(
                    (section) => section.key === step.filterSidebarSectionKey,
                  )?.title ?? ""}
                </p>
                <div className="trieve-inference-filters-step-row">
                  {filterOptions[step.filterSidebarSectionKey ?? ""]?.map(
                    (tag) => {
                      const currentFilterOption =
                        props.searchPageProps?.filterSidebarProps?.sections
                          .find(
                            (section) =>
                              section.key === step.filterSidebarSectionKey,
                          )
                          ?.options?.find((option) => option.tag === tag);

                      return (
                        <FilterButton
                          sectionKey={step.filterSidebarSectionKey ?? ""}
                          filterKey={tag}
                          label={currentFilterOption?.label ?? tag}
                          type={"single"}
                          onClick={() => {
                            stopGeneratingMessage();
                            clearConversation();
                            setCompletedSteps((prev) => {
                              const newCompletedSteps = { ...prev };
                              for (let j = index; j < steps.length; j++) {
                                newCompletedSteps[steps[j].title] = false;
                              }
                              return newCompletedSteps;
                            });
                          }}
                        />
                      );
                    },
                  )}
                </div>
                <div className="trieve-inference-filters-step-row">
                  {filterOptions[step.filterSidebarSectionKey ?? ""]?.map(
                    (tag) => {
                      const active =
                        selectedSidebarFilters[
                          step.filterSidebarSectionKey ?? ""
                        ]?.includes(tag);

                      const currentFilterOption =
                        props.searchPageProps?.filterSidebarProps?.sections
                          .find(
                            (section) =>
                              section.key === step.filterSidebarSectionKey,
                          )
                          ?.options?.find((option) => option.tag === tag);

                      return (
                        <div
                          data-active={active ? "true" : "false"}
                          className="trieve-inference-filters-step-tags-children"
                        >
                          <p className="trieve-inference-filters-step-tags-section-title">
                            {currentFilterOption?.child?.title ?? ""}
                          </p>
                          <div className="trieve-inference-filters-step-row">
                            {currentFilterOption?.child?.options?.map(
                              (childTagProp) => (
                                <FilterButton
                                  sectionKey={
                                    step.filterSidebarSectionKey ?? ""
                                  }
                                  filterKey={childTagProp.tag}
                                  label={childTagProp.label ?? childTagProp.tag}
                                  type={
                                    currentFilterOption?.child?.selectionType ??
                                    "single"
                                  }
                                  isChild={true}
                                  onClick={() => {
                                    stopGeneratingMessage();
                                    clearConversation();
                                    setCompletedSteps((prev) => {
                                      const newCompletedSteps = { ...prev };
                                      for (
                                        let j = index;
                                        j < steps.length;
                                        j++
                                      ) {
                                        newCompletedSteps[steps[j].title] =
                                          false;
                                      }
                                      return newCompletedSteps;
                                    });
                                  }}
                                />
                              ),
                            )}
                          </div>
                        </div>
                      );
                    },
                  )}
                </div>
              </div>

              {step.inputLabel && (
                <div className="trieve-text-input-container">
                  <label
                    htmlFor="text-input"
                    className="trieve-text-input-label"
                  >
                    {step.inputLabel}
                  </label>
                  <div className="trieve-text-input-textarea-container">
                    <textarea
                      name="text-input"
                      rows={4}
                      className="trieve-text-input-textarea"
                      disabled={completedSteps[step.title]}
                      placeholder={step.placeholder}
                      value={textFields[step.title]?.inputValue}
                      onChange={(e) => {
                        setTextFields((prev) => ({
                          ...prev,
                          [step.title]: {
                            ...prev[step.title],
                            inputValue: e.target.value,
                          },
                        }));
                      }}
                    />
                  </div>
                </div>
              )}

              <div className="trieve-text-button-container">
                <button
                  type="button"
                  className="trieve-text-input-button"
                  disabled={!textFields[step.title]?.inputValue}
                  onClick={() => {
                    setCompletedSteps((prev) => ({
                      ...prev,
                      [step.title]: true,
                    }));
                  }}
                >
                  Next
                  <div>
                    <i className="fa-solid fa-arrow-right"></i>
                  </div>
                </button>
              </div>
            </div>

            <div
              className="trieve-inference-filters-search-modal-container"
              data-prev-complete={
                index == 0 || completedSteps[steps[index - 1].title]
                  ? "true"
                  : "false"
              }
              data-input-field-type={step.type}
            >
              {step.type === "search_modal" && (
                <div className="trieve-inference-filters-search-modal">
                  <ModalContainer />
                </div>
              )}
            </div>
          </div>
          <div
            className="trieve-inference-filters-step-loading-container"
            data-loading-state={loadingStates[step.title] ?? "idle"}
          >
            <LoadingIcon className="loading" />
            <p className="trieve-loading-text">{loadingStates[step.title]}</p>
          </div>
        </div>
      ))}
    </div>
  );
};
