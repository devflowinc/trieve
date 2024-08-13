import { createQuery } from "@tanstack/solid-query";
import { getSearchQuery } from "../api/analytics";
import { Show, useContext } from "solid-js";
import { DatasetContext } from "../layouts/TopBarLayout";
import { JSONMetadata } from "shared/ui";

interface SingleQueryProps {
  queryId: string;
}
export const SingleQuery = (props: SingleQueryProps) => {
  const dataset = useContext(DatasetContext);

  const query = createQuery(() => ({
    queryKey: ["single_query", props.queryId],
    queryFn: () => {
      return getSearchQuery(dataset().dataset.id, props.queryId);
    },
  }));

  return (
    <div>
      <div>Single Query Info</div>
      <Show when={query.data}>{(data) => <JSONMetadata data={data()} />}</Show>
    </div>
  );
};
