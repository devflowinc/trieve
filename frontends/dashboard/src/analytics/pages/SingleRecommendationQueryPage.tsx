import { useParams } from "@solidjs/router";
import { SingleRecommendationQuery } from "../components/SingleRecommendationInfo";

export const SingleRecommendationQueryPage = () => {
  const params = useParams();
  return (
    <div>
      <SingleRecommendationQuery queryId={params.queryId} />
    </div>
  );
};
