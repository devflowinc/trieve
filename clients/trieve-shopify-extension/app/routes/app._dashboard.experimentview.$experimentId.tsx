import {
  Badge,
  Box,
  Button,
  Card,
  Frame,
  Layout,
  Page,
  Spinner,
  Text,
  Toast,
  BlockStack,
  InlineGrid,
  Link as PolarisLink,
} from "@shopify/polaris";
import { ArrowLeftIcon } from "@shopify/polaris-icons";
import { Link as RemixLink, useParams } from "@remix-run/react";
import { useContext, useEffect, useState } from "react";
import {
  TrieveContext,
} from "app/context/trieveContext";
import {
  Experiment,
  AnalyticsQueryBuilder,
} from "trieve-ts-sdk";

interface VariantStat {
  variant_name: string;
  user_count: number;
  conversion_rate?: number;
  uplift?: number;
}

export default function ExperimentReportPage() {
  const params = useParams();
  const experimentId = params.experimentId as string;

  const { trieve } = useContext(TrieveContext);

  const [experiment, setExperiment] = useState<Experiment | null>(null);
  const [variantStats, setVariantStats] = useState<VariantStat[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [toastActive, setToastActive] = useState(false);
  const [toastMessage, setToastMessage] = useState("");
  const [toastIsError, setToastIsError] = useState(false);

  const showToast = (message: string, isError = false) => {
    setToastMessage(message);
    setToastIsError(isError);
    setToastActive(true);
  };
  const toastMarkup = toastActive ? (
    <Toast content={toastMessage} error={toastIsError} onDismiss={() => setToastActive(false)} />
  ) : null;

  useEffect(() => {
    const fetchReportData = async () => {
      if (!trieve || !experimentId) {
        setError("SDK not available or Experiment ID missing.");
        setIsLoading(false);
        return;
      }
      setIsLoading(true);
      setError(null);

      try {
        // 1. Fetch Experiment Details
        const allExperiments = await trieve.getExperiments();
        const currentExp = allExperiments.find(e => String(e.id) === experimentId);
        if (!currentExp) {
          throw new Error("Experiment not found.");
        }
        setExperiment(currentExp);

        // 2. Fetch User Counts per Variant - AnalyticsQuery construction
        const userCountsQuery = new AnalyticsQueryBuilder()
          .select("treatment_name", { alias: "variant_name" })
          .select("user_id", { aggregation: "COUNT", alias: "user_count", distinct: true })
          .from("experiment_user_assignments")
          .where({
            column: "experiment_id",
            operator: "=",
            value: experimentId,
          })
          .groupBy(["treatment_name"])
          .build();
        
        const userCountsResult = await trieve.getAnalytics(userCountsQuery);
        
        if (Array.isArray(userCountsResult)) {
            setVariantStats(userCountsResult as VariantStat[]);
        } else {
            console.warn("User counts data is not an array:", userCountsResult);
            showToast("Could not parse user count data from analytics.", true);
            setVariantStats([]);
        }

      } catch (err) {
        console.error("Failed to fetch experiment report data:", err);
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        showToast(`Error loading report: ${errorMessage}`, true);
      } finally {
        setIsLoading(false);
      }
    };

    fetchReportData();
  }, [trieve, experimentId]);

  if (isLoading) {
    return (
      <Page title="Experiment Report">
        <Layout>
          <Layout.Section>
            <Spinner accessibilityLabel="Loading report data" size="large" />
          </Layout.Section>
        </Layout>
      </Page>
    );
  }

  if (error || !experiment) {
    return (
      <Page title="Error">
         <Layout>
            <Layout.Section>
                <BlockStack gap="400">
                    <RemixLink to="/app/experiments" style={{ textDecoration: 'none' }}>
                        <Button icon={ArrowLeftIcon}>Back to Experiments</Button>
                    </RemixLink>
                    <Card>
                        <Box padding="400">
                        <Text variant="headingLg" as="h2">Failed to load report</Text>
                        <Text as="p">{error || "Experiment not found."}</Text>
                        </Box>
                    </Card>
                </BlockStack>
            </Layout.Section>
        </Layout>
      </Page>
    );
  }

  const controlVariant = variantStats.find(v => v.variant_name === experiment.control_name);
  const treatmentVariant = variantStats.find(v => v.variant_name === experiment.t1_name);

  return (
    <Frame>
      <Page 
        title={`Report: ${experiment.name}`}
        backAction={{
            content: "Back to Experiments",
            url: "/app/experiments",
        }}
       >
        <Layout>
          <Layout.Section>
            <BlockStack gap="500">
              <Card>
                <Box padding="400">
                    <Text variant="headingMd" as="h2">Experiment Configuration</Text>
                    <BlockStack gap="200">
                        <Text as="p"><strong>Control:</strong> {experiment.control_name} ({(experiment.control_split * 100).toFixed(0)}%)</Text>
                        <Text as="p"><strong>Treatment:</strong> {experiment.t1_name} ({(experiment.t1_split * 100).toFixed(0)}%)</Text>
                        <Text as="p"><strong>Area:</strong> {experiment.area || "N/A"}</Text>
                        <Text as="p"><strong>ID:</strong> {String(experiment.id)}</Text>
                        <Text as="p"><strong>Created:</strong> {new Date(experiment.created_at).toLocaleDateString()}</Text>
                    </BlockStack>
                </Box>
              </Card>

              <InlineGrid columns={{ xs: 1, sm: 2, md: 2, lg: 4, xl: 4 }} gap="400">
                <Card>
                    <Box padding="400">
                        <Text variant="headingMd" as="h3">Control Users</Text>
                        <Text variant="headingLg" as="p">{controlVariant?.user_count?.toLocaleString() || "0"}</Text>
                    </Box>
                </Card>
                <Card>
                    <Box padding="400">
                        <Text variant="headingMd" as="h3">Treatment Users</Text>
                        <Text variant="headingLg" as="p">{treatmentVariant?.user_count?.toLocaleString() || "0"}</Text>
                    </Box>
                </Card>
                <Card>
                    <Box padding="400">
                         <Text variant="headingMd" as="h3">Control CR</Text>
                        <Text variant="headingLg" as="p">--%</Text>
                         <Text as="p" tone="subdued">Conversions: N/A</Text>
                    </Box>
                </Card>
                <Card>
                     <Box padding="400">
                        <Text variant="headingMd" as="h3">Treatment CR</Text>
                        <Text variant="headingLg" as="p">--%</Text>
                        <Text as="p" tone="subdued">Conversions: N/A</Text>
                    </Box>
                </Card>
              </InlineGrid>
              
              <Card>
                 <Box padding="400">
                    <Text variant="headingMd" as="h2">Further Analytics (Placeholders)</Text>
                    <BlockStack gap="200">
                        <Text as="p">Uplift: --%</Text>
                        <Text as="p">P-value (Significance): --</Text>
                        <Text as="p">Charts for users over time and conversion rates over time will be displayed here.</Text>
                    </BlockStack>
                </Box>
              </Card>

            </BlockStack>
          </Layout.Section>
        </Layout>
        {toastMarkup}
      </Page>
    </Frame>
  );
} 