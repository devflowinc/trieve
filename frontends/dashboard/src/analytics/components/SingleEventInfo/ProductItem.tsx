import { useContext, createResource, Show } from "solid-js";
import { getChunkQuery } from "../../../api/getItem";
import { DatasetContext } from "../../../contexts/DatasetContext";

type Props = {
  chunkId: string;
  index: number;
  position?: string;
};

export const ProductItem = (props: Props) => {
  const dataset = useContext(DatasetContext);

  const [item] = createResource(() =>
    getChunkQuery(dataset.datasetId(), props.chunkId),
  );

  return (
    <div class="w-[10rem] overflow-hidden rounded-lg bg-white shadow-md">
      {/* Product Image */}
      <div class="bg-gray-100">
        <Show when={item()?.image_urls?.[0]}>
          <img
            src={item()?.image_urls?.[0] ?? ""}
            alt="K2 Split Bean Splitboard"
            class="object-cover"
          />
        </Show>
      </div>
      <div class="p-4">
        <h2 class="mb-1 font-semibold text-gray-800">
          <Show when={(item()?.metadata as { title: string })?.title}>
            {(item()?.metadata as { title: string })?.title}
          </Show>
        </h2>

        <div class="mt-2 flex items-center justify-between">
          <div>
            <p class="text-gray-900">
              <Show when={item()?.num_value}>
                ${(item()?.num_value as number).toFixed(2)}
              </Show>
            </p>
            <p class="text-sm text-gray-500">
              <Show when={props.position}>
                Clicked at position {props.position}
              </Show>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
