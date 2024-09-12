import { For, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { createQuery } from "@tanstack/solid-query";
import { MagicBox } from "../../components/MagicBox";

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
      <MagicBox heightKey="big-tes" query={numQuery}>
        {(d) => (
          <div>
            testing
            <For each={d}>
              {(num) => <div class="m-3 bg-orange-700">{num}</div>}
            </For>
          </div>
        )}
      </MagicBox>
    </div>
  );
};
