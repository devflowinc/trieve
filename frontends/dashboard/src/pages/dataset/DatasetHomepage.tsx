import { useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";

export const DatasetHomepage = () => {
  const { datasetId } = useContext(DatasetContext);
  return (
    <div>
      <div>Dataset Homepage</div>
      <div class="m-3 bg-orange-700">ID: {datasetId}</div>
    </div>
  );
};
