import { createFileRoute } from "@tanstack/react-router";
import { Recommendations } from "../../../src/recommendations/Recommendations.tsx";

export const Route = createFileRoute("/recs")({
  component: RouteComponent,
});

function RouteComponent() {
  const baseUrl = import.meta.env.VITE_API_BASE_URL;
  const datasetId = import.meta.env.VITE_DATASET_ID;
  const apiKey = import.meta.env.VITE_API_KEY;
  return (
    <div className="grid">
      <Recommendations
        productId="50294006579481"
        apiKey={apiKey}
        baseUrl={baseUrl}
        datasetId={datasetId}
      />
    </div>
  );
}
