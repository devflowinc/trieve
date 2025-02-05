/* eslint-disable @typescript-eslint/no-explicit-any */
import { Match, Show, Switch, createSignal, useContext, For } from "solid-js";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { BiSolidFile } from "solid-icons/bi";
import { JsonInput, Tooltip } from "shared/ui";
import { BsInfoCircle } from "solid-icons/bs";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";

interface RequestBody {
  base64_file: string;
  file_name: string;
  link?: string;
  tag_set?: string[];
  split_delimiters: string[];
  target_splits_per_chunk: number;
  rebalance_chunks: boolean;
  group_tracking_id?: string;
  metadata: any;
  time_stamp?: string;
  split_avg: boolean;
  pdf2md_options?: {
    use_pdf2md_ocr: boolean;
    system_prompt?: string;
    split_headings?: boolean;
  };
}

export const UploadFile = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const dashboardUrl = import.meta.env.VITE_DASHBOARD_URL as string;

  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const [files, setFiles] = createSignal<File[]>([]);
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [link, setLink] = createSignal("");
  const [metadata, setMetadata] = createSignal<any>(undefined);
  const [tagSet, setTagSet] = createSignal("");
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [errorText, setErrorText] = createSignal("");
  const [submitted, setSubmitted] = createSignal(false);
  const [timestamp, setTimestamp] = createSignal("");
  const [splitDelimiters, setSplitDelimiters] = createSignal([".", "?", "\\n"]);
  const [targetSplitsPerChunk, setTargetSplitsPerChunk] = createSignal(20);
  const [rebalanceChunks, setRebalanceChunks] = createSignal(false);
  const [useGptChunking, setUseGptChunking] = createSignal(false);
  const [enableSplitAvg, setEnableSplitAvg] = createSignal(false);
  const [useHeadingBasedChunking, setUseHeadingBasedChunking] =
    createSignal(false);
  const [groupTrackingId, setGroupTrackingId] = createSignal("");
  const [systemPrompt, setSystemPrompt] = createSignal("");

  const [showFileInput, setShowFileInput] = createSignal(true);
  const [showFolderInput, setShowFolderInput] = createSignal(false);

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const items = e.dataTransfer?.items;
    if (items) {
      const traversePromises: Promise<File[]>[] = [];
      for (const item of items) {
        const entry = item.webkitGetAsEntry();
        if (entry) {
          traversePromises.push(traverseFileTree(entry));
        }
      }
      Promise.all(traversePromises)
        .then((results) => {
          const allFiles = results.flat();
          setFiles((prevFiles) => [...prevFiles, ...allFiles]);
        })
        .catch((error) => {
          console.error("Error traversing folder:", error);
        });
    }
  };

  const traverseFileTree = (
    item: FileSystemEntry,
    path = "",
  ): Promise<File[]> => {
    return new Promise<File[]>((resolve, reject) => {
      if (item.isFile) {
        (item as FileSystemFileEntry).file((file) => {
          resolve([file]);
        }, reject);
      } else if (item.isDirectory) {
        const dirReader = (item as FileSystemDirectoryEntry).createReader();
        const entries: FileSystemEntry[] = [];

        const readEntries = () => {
          dirReader.readEntries((newEntries) => {
            if (newEntries.length) {
              entries.push(...newEntries);
              readEntries();
            } else {
              Promise.all(
                entries.map((entry) =>
                  traverseFileTree(entry, path + item.name + "/"),
                ),
              )
                .then((nestedFiles) => {
                  const allFiles = nestedFiles.flat();
                  resolve(allFiles);
                })
                .catch(reject);
            }
          }, reject);
        };
        readEntries();
      } else {
        resolve([]);
      }
    });
  };

  const handleDirectUpload = (e: Event & { target: HTMLInputElement }) => {
    e.preventDefault();
    e.stopPropagation();
    const uploadedFiles = e.target.files ? Array.from(e.target.files) : [];
    setFiles(uploadedFiles);
  };

  const uploadFiles = async (e: Event) => {
    e.preventDefault();
    e.stopPropagation();
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    if (files().length === 0) {
      setErrorText("Please select files to upload");
      setIsSubmitting(false);
      return;
    }

    setErrorText("");
    setIsSubmitting(true);
    const toBase64 = (file: File) =>
      new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = reject;
      });

    const requestBodyTemplate: Omit<RequestBody, "base64_file" | "file_name"> =
      {
        link: link() === "" ? undefined : link(),
        tag_set:
          tagSet().split(",").length > 0 ? tagSet().split(",") : undefined,
        split_delimiters: splitDelimiters(),
        target_splits_per_chunk: targetSplitsPerChunk(),
        rebalance_chunks: rebalanceChunks(),
        split_avg: enableSplitAvg(),
        pdf2md_options: {
          use_pdf2md_ocr: useGptChunking(),
          split_headings: useHeadingBasedChunking(),
          system_prompt: systemPrompt(),
        },
        group_tracking_id:
          groupTrackingId() === "" ? undefined : groupTrackingId(),
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        metadata: metadata(),
        time_stamp: timestamp() ? timestamp() + " 00:00:00" : undefined,
      };

    const uploadFilePromises = files().map(async (file) => {
      let base64File = await toBase64(file);
      base64File = base64File
        .split(",")[1]
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=+$/, "");
      const requestBody: RequestBody = {
        ...requestBodyTemplate,
        base64_file: base64File,
        file_name: file.name,
      };

      return fetch(`${apiHost}/file`, {
        method: "POST",
        headers: {
          "X-API-version": "2.0",
          "Content-Type": "application/json",
          "TR-Dataset": currentDataset.dataset.id,
        },
        credentials: "include",
        body: JSON.stringify(requestBody),
      });
    });

    try {
      const responses = await Promise.all(uploadFilePromises);
      const allSuccess = responses.every((response) => response.ok);
      if (allSuccess) {
        setMetadata(undefined);
        setFiles([]);
        setLink("");
        setTagSet("");
        setIsSubmitting(false);
        setSubmitted(true);
      } else {
        setErrorText("Some files failed to upload. Please try again.");
        setIsSubmitting(false);
      }
    } catch (error) {
      setErrorText("Something went wrong. Please try again.");
      console.log(error);
      setIsSubmitting(false);
    }
  };

  return (
    <>
      <div class="text-center text-red-500">{errorText()}</div>
      <Show when={submitted()}>
        <div class="text-center font-semibold text-green-500">
          Your file has been uploaded successfully. Checkout your{" "}
          <a
            class="underline"
            href={`/group?dataset=${$dataset?.()?.dataset.id}`}
          >
            groups
          </a>{" "}
          to see it. There may be a delay before it appears. Monitor your chunk
          count in the{" "}
          <a class="underline" href={dashboardUrl}>
            dashboard
          </a>{" "}
          for an increase to check progress.
        </div>
      </Show>
      <div class="my-4 flex w-full flex-col gap-y-3">
        <div>Group tracking_id</div>
        <input
          type="url"
          placeholder="optional"
          value={groupTrackingId()}
          onInput={(e) => setGroupTrackingId(e.target.value)}
          class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
        />
        <div>Link to file</div>
        <input
          type="url"
          placeholder="optional"
          value={link()}
          onInput={(e) => setLink(e.target.value)}
          class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
        />
        <div>Tag Set</div>
        <input
          type="text"
          placeholder="optional - separate with commas"
          value={tagSet()}
          onInput={(e) => setTagSet(e.target.value)}
          class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
        />
        <button
          class="flex flex-row items-center gap-2"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            setShowAdvanced(!showAdvanced());
          }}
        >
          <Switch>
            <Match when={!showAdvanced()}>
              <FiChevronDown />
            </Match>
            <Match when={showAdvanced()}>
              <FiChevronUp />
            </Match>
          </Switch>
          Advanced options
        </button>
        <Show when={showAdvanced()}>
          <div class="ml-4 flex flex-col space-y-2">
            <div>Metadata </div>
            <JsonInput
              onValueChange={(j) => {
                setErrorText("");
                setMetadata(j);
              }}
              value={metadata}
              onError={(e) => setErrorText(`Error in Metadata: ${e}`)}
            />
            <div>Date</div>
            <input
              type="date"
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              onInput={(e) => {
                setTimestamp(e.currentTarget.value);
              }}
            />
            <div class="flex flex-col space-y-2">
              <div class="flex flex-row items-center space-x-2">
                <div>Split Delimiters</div>
                <Tooltip
                  body={<BsInfoCircle />}
                  tooltipText="Split delimiters is an optional field which allows you to specify the delimiters to use when splitting the file before chunking the text. If not specified, the default [.!?\n] are used to split into sentences. However, you may want to use spaces or other delimiters."
                />
              </div>
              <input
                type="text"
                placeholder="optional - separate with commas"
                value={splitDelimiters().join(",")}
                onInput={(e) => setSplitDelimiters(e.target.value.split(","))}
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
            </div>
            <div class="flex flex-row items-center space-x-2">
              <div>Target Splits Per Chunk</div>
              <Tooltip
                body={<BsInfoCircle />}
                tooltipText="Target splits per chunk. This is an optional field which allows you to specify the number of splits you want per chunk. If not specified, the default 20 is used. However, you may want to use a different number."
              />
            </div>
            <input
              type="number"
              placeholder="optional"
              value={targetSplitsPerChunk()}
              onChange={(e) =>
                setTargetSplitsPerChunk(parseInt(e.target.value))
              }
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <div class="flex flex-row items-center space-x-2">
              <div>Rebalance Chunks</div>
              <Tooltip
                body={<BsInfoCircle />}
                tooltipText="Balance chunks. If set to true, Trieve will evenly distribute remainder splits across chunks such that 46 splits with a target_splits_per_chunk of 20 will result in 3 chunks with 22 splits each."
              />
            </div>
            <input
              type="checkbox"
              checked={rebalanceChunks()}
              onInput={(e) => setRebalanceChunks(e.currentTarget.checked)}
              class="h-4 w-4 rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <div class="flex flex-row items-center space-x-2">
              <div>Use gpt4o chunking</div>
              <Tooltip
                body={<BsInfoCircle />}
                tooltipText="Use gpt4o chunking. If set to true, Trieve will use the gpt4o model to chunk the document if it is a pdf file. This is an experimental feature and may not work as expected."
              />
            </div>
            <input
              type="checkbox"
              checked={useGptChunking()}
              onInput={(e) => setUseGptChunking(e.currentTarget.checked)}
              class="h-4 w-4 rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <div class="flex flex-row items-center space-x-2">
              <div>Heading Based Chunking</div>
              <Tooltip
                body={<BsInfoCircle />}
                tooltipText="If set to true, Trieve will use the headings in the document to chunk the text."
              />
            </div>
            <input
              type="checkbox"
              checked={useHeadingBasedChunking()}
              onInput={(e) =>
                setUseHeadingBasedChunking(e.currentTarget.checked)
              }
              class="h-4 w-4 rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <div class="flex flex-row items-center space-x-2">
              <div>Join as single chunk</div>
              <Tooltip
                body={<BsInfoCircle />}
                tooltipText="If set to true, Trieve will create multiple embeddings for your chunk and average the resulting vectors as a single chunk."
              />
            </div>
            <input
              type="checkbox"
              checked={enableSplitAvg()}
              onInput={(e) => setEnableSplitAvg(e.currentTarget.checked)}
              class="h-4 w-4 rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <div class="flex flex-col space-y-2">
              <div class="flex flex-row items-center space-x-2">
                <div>System Prompt</div>
                <Tooltip
                  body={<BsInfoCircle />}
                  tooltipText="System prompt to use when chunking. This is an optional field which allows you to specify the system prompt to use when chunking the text. If not specified, the default system prompt is used. However, you may want to use a different system prompt."
                />
              </div>
              <textarea
                placeholder="optional system prompt to use when chunking"
                value={systemPrompt()}
                onInput={(e) => setSystemPrompt(e.target.value)}
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
            </div>
          </div>
        </Show>
        <div class="m-1 mb-1 flex flex-row gap-2">
          <button
            class={`rounded border-2 border-magenta p-2 px-4 font-semibold ${
              showFileInput()
                ? "bg-magenta-600 text-white"
                : "text-magenta hover:bg-magenta-500 hover:text-white"
            }`}
            onClick={() => {
              setFiles([]);
              setShowFileInput(true);
              setShowFolderInput(false);
            }}
          >
            Select Files
          </button>
          <button
            class={`rounded border-2 border-magenta p-2 px-4 font-semibold ${
              showFolderInput()
                ? "bg-magenta-600 text-white"
                : "text-magenta hover:bg-magenta-500 hover:text-white"
            }`}
            onClick={() => {
              setFiles([]);
              setShowFolderInput(true);
              setShowFileInput(false);
            }}
          >
            Select Folders
          </button>
        </div>
        <Show when={showFileInput() || showFolderInput()}>
          <label
            for="dropzone-file"
            class="dark:hover:bg-bray-800 flex max-h-96 min-h-64 w-full cursor-pointer flex-col items-center justify-center overflow-auto rounded-lg border-2 border-dashed border-gray-300 bg-neutral-100 hover:bg-neutral-200 dark:border-gray-600 dark:bg-neutral-700 dark:hover:border-gray-500 dark:hover:bg-gray-600"
            onDragOver={(e) => {
              e.preventDefault();
              e.stopPropagation();
            }}
            onDrop={handleDrop}
          >
            <div class="flex w-full flex-col items-center justify-center pb-6 pt-6">
              <Show when={files().length === 0}>
                <svg
                  fill="currentColor"
                  stroke-width="0"
                  style={{ overflow: "visible", color: "currentColor" }}
                  viewBox="0 0 16 16"
                  class="mb-3 h-10 w-10 text-gray-400"
                  height="1em"
                  width="1em"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    fill-rule="evenodd"
                    d="M4.406 1.342A5.53 5.53 0 018 0c2.69 0 4.923 2 5.166 4.579C14.758 4.804 16 6.137 16 7.773 16 9.569 14.502 11 12.687 11H10a.5.5 0 010-1h2.688C13.979 10 15 8.988 15 7.773c0-1.216-1.02-2.228-2.313-2.228h-.5v-.5C12.188 2.825 10.328 1 8 1a4.53 4.53 0 00-2.941 1.1c-.757.652-1.153 1.438-1.153 2.055v.448l-.445.049C2.064 4.805 1 5.952 1 7.318 1 8.785 2.23 10 3.781 10H6a.5.5 0 010 1H3.781C1.708 11 0 9.366 0 7.318c0-1.763 1.266-3.223 2.942-3.593.143-.863.698-1.723 1.464-2.383z"
                  />
                  <path
                    fill-rule="evenodd"
                    d="M7.646 4.146a.5.5 0 01.708 0l3 3a.5.5 0 01-.708.708L8.5 5.707V14.5a.5.5 0 01-1 0V5.707L5.354 7.854a.5.5 0 11-.708-.708l3-3z"
                  />
                </svg>
                <p class="mb-2 text-sm text-gray-500 dark:text-gray-400">
                  <span class="font-semibold">
                    Click to upload {showFileInput() ? "files" : "a folder"}
                  </span>{" "}
                  or drag and drop
                </p>
              </Show>
              <Show when={files().length > 0}>
                <div class="flex w-full flex-col items-center">
                  <For each={files()}>
                    {(file) => (
                      <div class="my-1 flex w-full items-center justify-center">
                        <BiSolidFile
                          classList={{ "mr-1": true }}
                          color="#6b7280"
                          fill="#6b7280"
                        />
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                          <span class="font-semibold">{file.name}</span>
                        </p>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
            {showFileInput() ? (
              <input
                id="dropzone-file"
                type="file"
                class="hidden"
                multiple
                onChange={handleDirectUpload}
              />
            ) : (
              <input
                id="dropzone-file"
                type="file"
                class="hidden"
                multiple
                {...({
                  webkitdirectory: true,
                } as any)}
                onChange={handleDirectUpload}
              />
            )}
          </label>
        </Show>
        <div class="flex flex-row items-center space-x-2">
          <button
            class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            type="submit"
            disabled={isSubmitting()}
            onClick={(e) => void uploadFiles(e)}
          >
            <Show when={!isSubmitting()}>Upload and Chunk Files</Show>
            <Show when={isSubmitting()}>
              <div class="animate-pulse">Uploading...</div>
            </Show>
          </button>
        </div>
      </div>
    </>
  );
};
