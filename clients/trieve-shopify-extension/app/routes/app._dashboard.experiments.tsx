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
  SkeletonPage,
  SkeletonBodyText,
  Toast,
  Frame,
  ActionList,
  ResourceItem,
  ResourceList,
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
import { TrieveContext } from "app/context/trieveContext";
import { Link, useNavigate, Form, useFetcher, useLoaderData } from "@remix-run/react";

export default function Experiments() {
  const { trieve } = useContext(TrieveContext);
  const navigate = useNavigate();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [experiments, setExperiments] = useState<Experiment[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [currentExperiment, setCurrentExperiment] = useState<Experiment | null>(null);
  const [experimentName, setExperimentName] = useState("");
  const [experimentArea, setExperimentArea] = useState("Global Search");
  const [controlName, setControlName] = useState("Control");
  const [treatmentName, setTreatmentName] = useState("Treatment 1");
  const [treatmentSplit, setTreatmentSplit] = useState(50);

  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [experimentToDeleteId, setExperimentToDeleteId] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const [toastActive, setToastActive] = useState(false);
  const [toastMessage, setToastMessage] = useState("");
  const [toastIsError, setToastIsError] = useState(false);

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
    setControlName("Control");
    setTreatmentName("Treatment 1");
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
    setControlName(experiment.control_name);
    setTreatmentName(experiment.t1_name);
    setTreatmentSplit(experiment.t1_split * 100);
    setIsModalOpen(true);
  };

  const handleModalClose = () => {
    if (isSubmitting) return;
    setIsModalOpen(false);
    resetForm();
  };

  const handleSubmitExperiment = useCallback(async () => {
    if (!trieve) {
      showToast("Trieve SDK is not initialized.", true);
      return;
    }
    if (!experimentName || !controlName || !treatmentName) {
      showToast("Please fill in all required name fields.", true);
      return;
    }

    setIsSubmitting(true);

    const currentConfigFromForm: ExperimentConfig = {
        area: experimentArea,
        control_name: controlName,
        control_split: parseFloat(((100 - treatmentSplit) / 100).toFixed(4)),
        t1_name: treatmentName,
        t1_split: parseFloat((treatmentSplit / 100).toFixed(4)),
    };

    try {
      if (currentExperiment && currentExperiment.id) {
        let updatePayload: UpdateExperimentReqBody = { id: String(currentExperiment.id) };

        if (experimentName !== currentExperiment.name) {
            updatePayload.name = experimentName;
        }

        const configChanged = 
            currentConfigFromForm.area !== (currentExperiment.area || "Global Search") ||
            currentConfigFromForm.control_name !== currentExperiment.control_name ||
            currentConfigFromForm.control_split !== currentExperiment.control_split ||
            currentConfigFromForm.t1_name !== currentExperiment.t1_name ||
            currentConfigFromForm.t1_split !== currentExperiment.t1_split;

        if (configChanged) {
            updatePayload.experiment_config = currentConfigFromForm;
        }

        if (!updatePayload.name && !updatePayload.experiment_config) {
            showToast("No changes detected.", false);
            setIsSubmitting(false);
            handleModalClose();
            return; 
        }
        const updatedExperiment = await trieve.updateExperiment(updatePayload);
        setExperiments((prev) =>
          prev.map((exp) => (exp.id === updatedExperiment.id ? {...updatedExperiment, id: String(updatedExperiment.id), area: experimentArea } : exp)),
        );
        showToast("Experiment updated successfully!");
      } else {
        const payload: CreateExperimentReqBody = {
          name: experimentName,
          experiment_config: currentConfigFromForm,
        };
        const newApiExperiment = await trieve.createExperiment(payload);
        setExperiments((prev) => [...prev, {...newApiExperiment, id: String(newApiExperiment.id), area: experimentArea}]);
        showToast("Experiment created successfully!");
      }
      handleModalClose();
    } catch (error) {
      console.error("Failed to save experiment:", error);
      showToast(`Failed to save experiment: ${error instanceof Error ? error.message : String(error)}`, true);
    } finally {
      setIsSubmitting(false);
    }
  }, [
    trieve,
    currentExperiment,
    experimentName,
    experimentArea,
    controlName,
    treatmentName,
    treatmentSplit,
    handleModalClose,
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
    if (!trieve || !experimentToDeleteId) {
      showToast("Cannot delete experiment: SDK not available or no experiment selected.", true);
      return;
    }
    setIsDeleting(true);
    try {
      await trieve.deleteExperiment(experimentToDeleteId);
      setExperiments((prev) => prev.filter((exp) => exp.id !== experimentToDeleteId));
      showToast("Experiment deleted successfully!");
      handleCloseDeleteConfirmModal();
    } catch (error) {
      console.error("Failed to delete experiment:", error);
      showToast(`Failed to delete experiment: ${error instanceof Error ? error.message : String(error)}`, true);
    } finally {
      setIsDeleting(false);
    }
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
              isSubmitting ||
              !experimentName ||
              !controlName ||
              !treatmentName,
            loading: isSubmitting,
          }}
          secondaryActions={[
            {
              content: "Cancel",
              onAction: handleModalClose,
              disabled: isSubmitting,
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
              <TextField
                label="Control Group Name"
                value={controlName}
                onChange={setControlName}
                autoComplete="off"
                placeholder="e.g., Current PDP Layout"
                requiredIndicator
                disabled={isSubmitting}
              />
              <TextField
                label="Treatment Group Name"
                value={treatmentName}
                onChange={setTreatmentName}
                autoComplete="off"
                placeholder="e.g., New PDP Layout with Video"
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
            loading: isDeleting,
            disabled: isDeleting,
          }}
          secondaryActions={[
            {
              content: "Cancel",
              onAction: handleCloseDeleteConfirmModal,
              disabled: isDeleting,
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
