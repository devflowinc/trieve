import { For, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { createQuery } from "@tanstack/solid-query";
import { MagicSuspense } from "../../components/MagicBox";

const slowFetchRandomNumber = (delay: number): Promise<number[]> => {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve([23, 23, 24]);
    }, delay);
  });
};

export const DatasetHomepage = () => {
  const { datasetId } = useContext(DatasetContext);

  const numQuery = createQuery(() => ({
    queryKey: ["randomNumber"],
    queryFn: () => slowFetchRandomNumber(2000),
  }));

  return (
    <div class="p-4">
      <div>Dataset Homepage</div>
      <div class="m-3 bg-orange-700">ID: {datasetId}</div>
      <MagicSuspense skeletonKey="big-tes">
        <div>
          <For each={numQuery.data}>
            {(num) => <div class="m-3 bg-orange-700">{num}</div>}
          </For>
        </div>
      </MagicSuspense>
    </div>
  );
};
