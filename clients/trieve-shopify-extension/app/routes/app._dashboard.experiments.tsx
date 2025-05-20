import {
  Page,
  Layout,
  Card,
  Button,
  BlockStack,
  Text,
  Modal,
  TextField,
  RangeSlider,
  InlineStack,
  EmptyState,
  Box,
  ButtonGroup,
  Spinner,
  SkeletonBodyText,
  Toast,
  Frame,
  Select,
} from "@shopify/polaris";
import { useState, useCallback, useContext, useEffect } from "react";
import { PlusIcon, DeleteIcon, EditIcon, ViewIcon } from "@shopify/polaris-icons";
import {
  type CreateExperimentReqBody,
  type Experiment,
  type UpdateExperimentReqBody,
  type ExperimentConfig,
} from "trieve-ts-sdk";
import { TrieveContext, useTrieve } from "app/context/trieveContext";
import { Link, useNavigate, Form, useFetcher, useLoaderData, json } from "@remix-run/react";
import { ActionFunctionArgs, data } from "@remix-run/node";
import { authenticate } from "../shopify.server";
import { getAppMetafields, setAppMetafields } from "../queries/metafield";
import { validateTrieveAuth } from "app/auth";
import { sdkFromKey } from "app/auth";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";

const METAFIELD_KEY_AB_TESTS = "ab_tests";

interface AbTestMetafieldValue {
  pdpExperimentIds: string[];
  globalExperimentIds: string[];
}

async function updateShopAbTestMetafield(
  admin: any,
  experimentId: string,
  area: string,
  operation: "add" | "remove"
) {
  // 1. Get current metafield using getAppMetafields
  let abTestData: AbTestMetafieldValue | null = await getAppMetafields<AbTestMetafieldValue>(
    admin,
    METAFIELD_KEY_AB_TESTS
  );

  if (!abTestData) {
    abTestData = { pdpExperimentIds: [], globalExperimentIds: [] };
  }

  abTestData.pdpExperimentIds = abTestData.pdpExperimentIds || [];
  abTestData.globalExperimentIds = abTestData.globalExperimentIds || [];

  abTestData.pdpExperimentIds = abTestData.pdpExperimentIds.filter(id => id !== experimentId);
  abTestData.globalExperimentIds = abTestData.globalExperimentIds.filter(id => id !== experimentId);

  if (operation === "add") {
    if (area === "PDP") {
      abTestData.pdpExperimentIds.push(experimentId);
    } else if (area === "Global Search") {
      abTestData.globalExperimentIds.push(experimentId);
    } else {
      console.warn(`Unknown experiment area: ${area} for experiment ${experimentId}. Not adding to metafield.`);
    }
  }

  try {
    await setAppMetafields(admin, [
      {
        key: METAFIELD_KEY_AB_TESTS,
        value: JSON.stringify(abTestData),
        type: "json",
      },
    ]);
  } catch (error) {
    console.error("Error setting A/B test metafield via setAppMetafields:", error);
    throw new Error("Failed to update Shopify A/B test metafield using helper.");
  }
}

export async function action({ request, context }: ActionFunctionArgs) {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!
  )

  const formData = await request.formData();
  const intent = formData.get("intent") as string;
  
  try {
    if (intent === "createExperiment" || intent === "updateExperiment") {
      const name = formData.get("name") as string;
      const area = formData.get("area") as string;
      const controlName = formData.get("control_name") as string;
      const t1Name = formData.get("t1_name") as string;
      const controlSplit = parseFloat(formData.get("control_split") as string);
      const t1Split = parseFloat(formData.get("t1_split") as string);

      if (!name || !area || !controlName || !t1Name || isNaN(controlSplit) || isNaN(t1Split)) {
        return data({ error: "Missing required experiment fields." }, { status: 400 });
      }

      const experimentConfig: ExperimentConfig = {
        area,
        control_name: controlName,
        control_split: controlSplit,
        t1_name: t1Name,
        t1_split: t1Split,
      };

      let savedExperiment: Experiment;

      if (intent === "createExperiment") {
        const payload: CreateExperimentReqBody = { name, experiment_config: experimentConfig };
        savedExperiment = await trieve.createExperiment(payload);
        await updateShopAbTestMetafield(fetcher, String(savedExperiment.id), area, "add");
        return data({ experiment: savedExperiment, intent: "createExperiment" });
      } else {
        const experimentId = formData.get("experimentId") as string;
        if (!experimentId) return data({ error: "Experiment ID is required for update."}, {status: 400});
        
        const updatePayload: UpdateExperimentReqBody = { id: experimentId };
        updatePayload.name = name; 
        updatePayload.experiment_config = experimentConfig; 

        savedExperiment = await trieve.updateExperiment(updatePayload);
        await updateShopAbTestMetafield(fetcher, String(savedExperiment.id), area, "add");
        return data({ experiment: savedExperiment, intent: "updateExperiment" });
      }
    } else if (intent === "deleteExperiment") {
      const experimentId = formData.get("experimentId") as string;
      if (!experimentId) return json({ error: "Experiment ID is required for delete."}, {status: 400});
      
      await trieve.deleteExperiment(experimentId);
      await updateShopAbTestMetafield(fetcher, experimentId, "", "remove");
      return data({ deletedExperimentId: experimentId });
    }
    return data({ error: "Invalid intent" }, { status: 400 });
  } catch (error: any) {
    console.error(`Failed to ${intent}:`, error);
    return json({ error: error.message || "Failed to process experiment action" }, { status: 500 });
  }
}

export default function Experiments() {
  const { trieve } = useContext(TrieveContext);
  const navigate = useNavigate();
  const fetcher = useFetcher();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [experiments, setExperiments] = useState<Experiment[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [currentExperiment, setCurrentExperiment] = useState<Experiment | null>(null);
  const [experimentName, setExperimentName] = useState("");
  const [experimentArea, setExperimentArea] = useState("Global Search");
  const [controlName, setControlName] = useState("Don't show");
  const [treatmentName, setTreatmentName] = useState("Show");
  const [treatmentSplit, setTreatmentSplit] = useState(50);

  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [experimentToDeleteId, setExperimentToDeleteId] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const [toastActive, setToastActive] = useState(false);
  const [toastMessage, setToastMessage] = useState("");
  const [toastIsError, setToastIsError] = useState(false);

  useEffect(() => {
    if (fetcher.state === "idle" && fetcher.data) {
      const data = fetcher.data as any;
      if (data.error) {
        showToast(`Error: ${data.error}`, true);
      } else if (data.experiment) {
        const savedExp = data.experiment as Experiment;
        if (data.intent === "createExperiment") {
          setExperiments((prev) => [...prev, { ...savedExp, id: String(savedExp.id), area: savedExp.area || "Global Search" }]);
          showToast("Experiment created successfully!");
        } else {
          setExperiments((prev) =>
            prev.map((exp) => (exp.id === String(savedExp.id) ? { ...savedExp, id: String(savedExp.id), area: savedExp.area || "Global Search" } : exp))
          );
          showToast("Experiment updated successfully!");
        }
        handleModalClose();
      } else if (data.deletedExperimentId) {
        setExperiments((prev) => prev.filter((exp) => exp.id !== data.deletedExperimentId));
        showToast("Experiment deleted successfully!");
        handleCloseDeleteConfirmModal();
      }
    }
    if (fetcher.state === "submitting" || fetcher.state === "loading") {
        setIsSubmitting(true);
    } else {
        setIsSubmitting(false);
        setIsDeleting(false);
    }
  }, [fetcher.state, fetcher.data]);

  const showToast = (message: string, isError = false) => {
    setToastMessage(message);
    setToastIsError(isError);
    setToastActive(true);
  };

  const toggleToastActive = () => setToastActive((active) => !active);

  const toastMarkup = toastActive ? (
    <Toast content={toastMessage} error={toastIsError} onDismiss={toggleToastActive} />
  ) : null;

  useEffect(() => {
    const fetchExperiments = async () => {
      setIsLoading(true);
      try {
        if (!trieve) {
            showToast("SDK not initialized.", true);
            setIsLoading(false);
            return;
        }
        const apiExperiments: Experiment[] = await trieve.getExperiments();
        setExperiments(apiExperiments.map(exp => ({...exp, id: String(exp.id), area: exp.area || "Global Search" }) ) );
      } catch (error) {
        console.error("Failed to fetch experiments:", error);
        showToast(`Failed to fetch experiments: ${error instanceof Error ? error.message : String(error)}`, true);
      } finally {
        setIsLoading(false);
      }
    };
    fetchExperiments();
  }, [trieve]);

  const resetForm = () => {
    setExperimentName("");
    setExperimentArea("Global Search");
    setControlName("Don't show");
    setTreatmentName("Show");
    setTreatmentSplit(50);
    setCurrentExperiment(null);
  };

  const handleOpenCreateModal = () => {
    resetForm();
    setIsModalOpen(true);
  };

  const handleOpenEditModal = (experiment: Experiment) => {
    setCurrentExperiment(experiment);
    setExperimentName(experiment.name);
    setExperimentArea(experiment.area || "Global Search");
    setControlName(experiment.control_name === "Show" || experiment.control_name === "Don't show" ? experiment.control_name : "Don't show");
    setTreatmentName(experiment.t1_name === "Show" || experiment.t1_name === "Don't show" ? experiment.t1_name : "Show");
    setTreatmentSplit(experiment.t1_split * 100);
    setIsModalOpen(true);
  };

  const handleModalClose = () => {
    if (isSubmitting) return;
    setIsModalOpen(false);
    resetForm();
  };

  const handleSubmitExperiment = useCallback(async () => {
    if (!experimentName || !controlName || !treatmentName) {
      showToast("Please fill in all required name fields.", true);
      return;
    }

    const formData = new FormData();
    formData.append("name", experimentName);
    formData.append("area", experimentArea);
    formData.append("control_name", controlName);
    formData.append("t1_name", treatmentName);
    formData.append("control_split", String(parseFloat(((100 - treatmentSplit) / 100).toFixed(4))));
    formData.append("t1_split", String(parseFloat((treatmentSplit / 100).toFixed(4))));

    if (currentExperiment && currentExperiment.id) {
      formData.append("intent", "updateExperiment");
      formData.append("experimentId", String(currentExperiment.id));
    } else {
      formData.append("intent", "createExperiment");
    }
    
    fetcher.submit(formData, { method: "POST" });
    handleModalClose();
  }, [
    experimentName,
    experimentArea,
    controlName,
    treatmentName,
    treatmentSplit,
    currentExperiment,
    fetcher,
  ]);

  const handleOpenDeleteConfirmModal = (id: string) => {
    setExperimentToDeleteId(id);
    setShowDeleteConfirmModal(true);
  };

  const handleCloseDeleteConfirmModal = () => {
    if (isDeleting) return;
    setShowDeleteConfirmModal(false);
    setExperimentToDeleteId(null);
  };

  const handleConfirmDelete = async () => {
    if (!experimentToDeleteId) {
      showToast("Cannot delete experiment: No experiment selected.", true);
      return;
    }

    const formData = new FormData();
    formData.append("intent", "deleteExperiment");
    formData.append("experimentId", experimentToDeleteId);

    fetcher.submit(formData, { method: "POST" });
  };

  const controlSplitPercentage = 100 - treatmentSplit;

  const renderExperimentsList = () => {
    if (experiments.length === 0 && !isLoading) {
      return (
        <Card>
          <EmptyState
            heading="No A/B tests found"
            action={{
              content: "New Experiment",
              onAction: handleOpenCreateModal,
              icon: PlusIcon,
              disabled: isSubmitting || isDeleting,
            }}
            image="https://cdn.shopify.com/s/files/1/0262/4074/files/empty-state.svg"
          >
            <p>
              Define and manage experiments to optimize search results, product
              pages, and more.
            </p>
          </EmptyState>
        </Card>
      );
    }

    return (
      <BlockStack gap="400">
        {experiments.map((exp) => (
          <Card key={exp.id}>
            <Box padding="400">
              <InlineStack
                align="center"
                blockAlign="center"
                wrap={false}
                gap="400"
              >
                <Box width="100%">
                  <BlockStack gap="100">
                    <Text variant="headingMd" as="h2">
                      {exp.name}
                    </Text>
                    <Text as="p" tone="subdued">
                      Area: {exp.area || "N/A"} | ID: {exp.id}
                    </Text>
                    <Text as="p">
                      {exp.control_name}: {(exp.control_split * 100).toFixed(0)}% vs.{" "}
                      {exp.t1_name}: {(exp.t1_split * 100).toFixed(0)}%
                    </Text>
                  </BlockStack>
                </Box>
                <ButtonGroup>
                  <Button icon={EditIcon} onClick={() => handleOpenEditModal(exp)} disabled={isSubmitting || isDeleting}>
                    Edit
                  </Button>
                  <Button icon={DeleteIcon} onClick={() => handleOpenDeleteConfirmModal(String(exp.id))} tone="critical" disabled={isSubmitting || isDeleting}>
                    Delete
                  </Button>
                  <Button onClick={() => navigate(`/app/experimentview/${exp.id}`)}>View Report</Button>
                </ButtonGroup>
              </InlineStack>
            </Box>
          </Card>
        ))}
      </BlockStack>
    );
  };

  if (isLoading) {
    return (
      <Page title="Experiments">
        <Layout>
          <Layout.Section>
            <Card>
              <Box padding="400">
                <Spinner accessibilityLabel="Loading experiments" size="large" />
              </Box>
            </Card>
          </Layout.Section>
          <Layout.Section variant="oneThird">
            <Card>
                <Box padding="400">
                    <SkeletonBodyText lines={5}/>
                </Box>
            </Card>
          </Layout.Section>
        </Layout>
      </Page>
    );
  }

  return (
    <Frame>
      <Page
        title="Experiments"
        primaryAction={{
          content: "New Experiment",
          icon: PlusIcon,
          onAction: handleOpenCreateModal,
          disabled: isSubmitting || isDeleting,
        }}
      >
        <Layout>
          <Layout.Section>{renderExperimentsList()}</Layout.Section>
        </Layout>

        <Modal
          open={isModalOpen}
          onClose={handleModalClose}
          title={currentExperiment ? "Edit Experiment" : "Create New Experiment"}
          primaryAction={{
            content: currentExperiment ? "Save Changes" : "Create Experiment",
            onAction: handleSubmitExperiment,
            disabled:
              (fetcher.state === 'submitting' || fetcher.state === 'loading') ||
              !experimentName ||
              !controlName ||
              !treatmentName,
            loading: (fetcher.state === 'submitting' || fetcher.state === 'loading'),
          }}
          secondaryActions={[
            {
              content: "Cancel",
              onAction: handleModalClose,
              disabled: (fetcher.state === 'submitting' || fetcher.state === 'loading'),
            },
          ]}
        >
          <Modal.Section>
            <BlockStack gap="400">
              <TextField
                label="Experiment Name"
                value={experimentName}
                onChange={setExperimentName}
                autoComplete="off"
                placeholder="e.g., Q4 PDP Improvement Test"
                requiredIndicator
                disabled={isSubmitting}
              />
              <Select
                label="Experiment Area"
                options={[
                  { label: "Global Search", value: "Global Search" },
                  { label: "PDP", value: "PDP" },
                ]}
                onChange={(value) => setExperimentArea(value)}
                value={experimentArea}
                helpText="Define where this experiment runs."
                disabled={isSubmitting}
              />
              <Text variant="headingMd" as="h3">
                Define Variants & Traffic Split
              </Text>
              <Select
                label="Control Group Name"
                options= {[
                  { label: "Show", value: "Show" },
                  { label: "Don't show", value: "Don't show" },
                ]}
                onChange={(value) => setControlName(value)}
                value={controlName}
                requiredIndicator
                disabled={isSubmitting}
              />
              <Select
                label="Treatment Group Name"
                options= {[
                  { label: "Show", value: "Show" },
                  { label: "Don't show", value: "Don't show" },
                ]}
                onChange={(value) => setTreatmentName(value)}
                value={treatmentName}
                requiredIndicator
                disabled={isSubmitting}
              />
              <BlockStack gap="200">
                <RangeSlider
                  label={`Treatment Group Traffic: ${treatmentSplit.toFixed(0)}%`}
                  value={treatmentSplit}
                  onChange={(value) =>
                    setTreatmentSplit(typeof value === "number" ? value : value[0])
                  }
                  min={0}
                  max={100}
                  step={1}
                  output
                  disabled={isSubmitting}
                />
                <Text as="p" tone="subdued">
                  Control Group Traffic: {controlSplitPercentage.toFixed(0)}%
                </Text>
              </BlockStack>
            </BlockStack>
          </Modal.Section>
        </Modal>

        <Modal
          open={showDeleteConfirmModal}
          onClose={handleCloseDeleteConfirmModal}
          title="Delete Experiment?"
          primaryAction={{
            content: "Delete",
            onAction: handleConfirmDelete,
            destructive: true,
            loading: (fetcher.state === 'submitting' || fetcher.state === 'loading'),
            disabled: (fetcher.state === 'submitting' || fetcher.state === 'loading'),
          }}
          secondaryActions={[
            {
              content: "Cancel",
              onAction: handleCloseDeleteConfirmModal,
              disabled: (fetcher.state === 'submitting' || fetcher.state === 'loading'),
            },
          ]}
        >
          <Modal.Section>
            <Text as="p">
              Are you sure you want to delete this experiment? This action cannot be undone.
            </Text>
          </Modal.Section>
        </Modal>
        {toastMarkup}
      </Page>
    </Frame>
  );
}
