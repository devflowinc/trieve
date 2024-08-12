import { createQuery } from "@tanstack/solid-query";
import { getSearchQuery } from "../api/analytics";
import { Show, useContext } from "solid-js";
import { DatasetContext } from "../layouts/TopBarLayout";

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
      <div>Single Query INfo</div>
      <Show when={query.data}>
        {(data) => <pre>{JSON.stringify(data(), null, 4)}</pre>}
      </Show>
    </div>
  );
};
