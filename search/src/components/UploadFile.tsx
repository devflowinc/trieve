/* eslint-disable @typescript-eslint/no-explicit-any */
import { Show, createSignal, useContext } from "solid-js";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { BiSolidFile } from "solid-icons/bi";
import { Tooltip } from "./Atoms/Tooltip";
import { BsInfoCircle } from "solid-icons/bs";

export const UploadFile = () => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const [file, setFile] = createSignal<File | undefined>();
  const [link, setLink] = createSignal("");
  const [tagSet, setTagSet] = createSignal("");
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [errorText, setErrorText] = createSignal("");
  const [submitted, setSubmitted] = createSignal(false);
  const [timestamp, setTimestamp] = createSignal("");
  const [splitDelimiters, setSplitDelimiters] = createSignal([".", "?", "\\n"]);
  const [targetSplitsPerChunk, setTargetSplitsPerChunk] = createSignal(20);
  const [rebalanceChunks, setRebalanceChunks] = createSignal(false);

  const handleDragUpload = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setFile(e.dataTransfer?.files[0] as any);
  };
  const handleDirectUpload = (e: Event & { target: HTMLInputElement }) => {
    e.preventDefault();
    e.stopPropagation();
    setFile(e.target.files ? (e.target.files[0] as any) : undefined);
  };
  const uploadFile = async (e: Event) => {
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    if (!file()) {
      setErrorText("Please select a file to upload");
      setIsSubmitting(false);
      return;
    }
    setErrorText("");
    e.preventDefault();
    setIsSubmitting(true);
    const toBase64 = (file: File) =>
      new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = () => resolve(reader.result);
        reader.onerror = reject;
      });

    let base64File = await toBase64(file() ?? new File([], ""));
    base64File = (base64File as string)
      .toString()
      .split(",")[1]
      .replace(/\+/g, "-") // Convert '+' to '-'
      .replace(/\//g, "_") // Convert '/' to '_'
      .replace(/=+$/, ""); // Remove ending '='

    const file_name = file()?.name;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const requestBody: any = {
      base64_file: base64File,
      file_name: file_name,
      link: link(),
      tag_set: tagSet().split(","),
      split_delimiters: splitDelimiters(),
      target_splits_per_chunk: targetSplitsPerChunk(),
      rebalance_chunks: rebalanceChunks(),
    };

    if (timestamp()) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      requestBody.time_stamp = timestamp() + " 00:00:00";
    }

    void fetch(`${apiHost}/file`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
      body: JSON.stringify(requestBody),
    }).then((response) => {
      if (!response.ok) {
        setIsSubmitting(false);
        setErrorText("Something went wrong. Please try again.");
        return;
      }
      void response.json().then(() => {
        setFile(undefined);
        setLink("");
        setTagSet("");
        setIsSubmitting(false);
        setSubmitted(true);
      });
    });
  };
  return (
    <>
      <div class="text-center text-red-500">{errorText()}</div>
      <Show when={submitted()}>
        <div class="text-center text-green-500">
          Your document has been uploaded and we will send you a notification
          when it has been processed.
        </div>
      </Show>
      <div class="my-4 flex w-full flex-col gap-y-3">
        <div class="flex flex-col space-y-2">
          <div>Link to file</div>
          <input
            type="url"
            placeholder="optional"
            value={link()}
            onInput={(e) => setLink(e.target.value)}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
        </div>
        <div class="flex flex-col space-y-2">
          <div>Tag Set</div>
          <input
            type="text"
            placeholder="optional - separate with commas"
            value={tagSet()}
            onInput={(e) => setTagSet(e.target.value)}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
          <div>Date</div>
          <input
            type="date"
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            onInput={(e) => {
              setTimestamp(e.currentTarget.value);
            }}
          />
        </div>
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
        <div class="flex flex-col space-y-2">
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
            onChange={(e) => setTargetSplitsPerChunk(parseInt(e.target.value))}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
        </div>
        <div class="flex flex-col space-y-2">
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
        </div>
        <label
          for="dropzone-file"
          class="dark:hover:bg-bray-800 flex h-64 w-full cursor-pointer flex-col items-center justify-center rounded-lg border-2 border-dashed border-gray-300 bg-neutral-100 hover:bg-neutral-200 dark:border-gray-600 dark:bg-neutral-700 dark:hover:border-gray-500 dark:hover:bg-gray-600"
          onDragOver={(e) => {
            e.preventDefault();
            e.stopPropagation();
          }}
          onDrop={handleDragUpload}
        >
          <div class="flex flex-col items-center justify-center pb-6 pt-5">
            <Show when={file() == undefined}>
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
                <span class="font-semibold">Click to upload</span> or drag and
                drop
              </p>
            </Show>
            <Show when={file() != undefined}>
              <div class="flex items-center">
                <BiSolidFile
                  classList={{ "mr-1 mb-2": true }}
                  color="#6b7280"
                  fill="#6b7280"
                />
                <p class="mb-2 text-sm text-gray-500 dark:text-gray-400">
                  <span class="font-semibold">{file()?.name}</span>
                </p>
              </div>
            </Show>
          </div>
          <input
            id="dropzone-file"
            type="file"
            class="hidden"
            onChange={handleDirectUpload}
          />
        </label>
        <div class="flex flex-row items-center space-x-2">
          <button
            class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            type="submit"
            disabled={isSubmitting()}
            onClick={(e) => void uploadFile(e)}
          >
            <Show when={!isSubmitting()}>Upload and Chunk New File</Show>
            <Show when={isSubmitting()}>
              <div class="animate-pulse">Uploading...</div>
            </Show>
          </button>
        </div>
      </div>
    </>
  );
};
