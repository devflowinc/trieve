/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  Accessor,
  createEffect,
  createSignal,
  For,
  Setter,
  Show,
  useContext,
} from "solid-js";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";

export interface ImageModalProps {
  showImageModal: Accessor<boolean>;
  setShowImageModal: Setter<boolean>;
  imgInformation: Accessor<{
    imgRangeStart: number;
    imgRangeEnd: number;
    imgRangePrefix: string;
  } | null>;
}

export const ImageModal = (props: ImageModalProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);
  const $currentDataset = datasetAndUserContext.currentDataset;

  const [signedImageUrlsHashmap, setSignedImageUrlsHashmap] = createSignal<
    Record<string, string>
  >({});

  createEffect(() => {
    const rangeArray = Array.from({
      length:
        (props.imgInformation()?.imgRangeEnd ?? 0) -
        (props.imgInformation()?.imgRangeStart ?? 0) +
        1,
    });

    rangeArray.forEach((_, i) => {
      const fileName = `${props.imgInformation()?.imgRangePrefix ?? ""}${
        (props.imgInformation()?.imgRangeStart ?? 0) + i
      }`;

      void fetch(`${apiHost}/file/get_signed_url/${fileName}.png`, {
        headers: {
          "X-API-version": "2.0",
          "TR-Dataset": $currentDataset?.()?.dataset.id ?? "",
        },
        credentials: "include",
      }).then((response) => {
        void response.json().then((data) => {
          const signedUrl = data.signed_url as string;
          setSignedImageUrlsHashmap((prev) => ({
            ...prev,
            [fileName]: signedUrl,
          }));
        });
      });
    });
  });

  return (
    <Show when={props.showImageModal()}>
      <FullScreenModal
        isOpen={props.showImageModal}
        setIsOpen={props.setShowImageModal}
      >
        <div class="flex max-h-[75vh] max-w-[75vw] flex-col space-y-2 overflow-auto">
          <For
            each={Array.from({
              length:
                (props.imgInformation()?.imgRangeEnd ?? 0) -
                (props.imgInformation()?.imgRangeStart ?? 0) +
                1,
            })}
          >
            {(_, i) => {
              const fileName = `${
                props.imgInformation()?.imgRangePrefix ?? ""
              }${
                // eslint-disable-next-line solid/reactivity
                (props.imgInformation()?.imgRangeStart ?? 0) + i()
              }`;
              const signedUrl = signedImageUrlsHashmap()[fileName] ?? "";

              return <img class="mx-auto my-auto" src={signedUrl} />;
            }}
          </For>
        </div>
      </FullScreenModal>
    </Show>
  );
};
