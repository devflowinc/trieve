import { useParams } from "@solidjs/router";
import { SingleEventQuery } from "../components/SingleEventInfo";

export const SingleEventQueryPage = () => {
  const params = useParams();
  return (
    <div>
      <SingleEventQuery queryId={params.queryId} />
    </div>
  );
};
