import { useParams } from "@solidjs/router";
import { SingleQuery } from "../components/SingleQueryInfo";

export const SingleQueryPage = () => {
  const params = useParams();
  return (
    <div>
      <SingleQuery queryId={params.queryId} />
    </div>
  );
};
