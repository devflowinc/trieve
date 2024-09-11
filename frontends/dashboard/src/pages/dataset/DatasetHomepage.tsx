import { useParams } from "@solidjs/router";

export const DatasetHomepage = () => {
  const datasetId = useParams().id;
  return (
    <div>
      <div>Dataset Homepage</div>
      <div class="m-3 h-[4000px] bg-orange-700">{datasetId}</div>
    </div>
  );
};
