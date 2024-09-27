import { useParams } from "@solidjs/router";
import { SingleRAGQuery } from "../components/SingleRagInfo";

export const SingleRAGQueryPage = () => {
  const params = useParams();
  return (
    <div>
      <SingleRAGQuery queryId={params.queryId} />
    </div>
  );
};
