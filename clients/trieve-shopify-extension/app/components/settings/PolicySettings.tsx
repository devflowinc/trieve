import {
  BlockStack,
  Box,
  Text,
  Card,
  Button,
  TextField,
  EmptyState,
  InlineStack,
  ResourceList,
  ResourceItem,
  ButtonGroup,
  Modal,
  InlineGrid,
} from "@shopify/polaris";
import { useEffect, useState } from "react";
import { Dataset } from "trieve-ts-sdk";
import { useSubmit } from "@remix-run/react";
import { PlusIcon, DeleteIcon, EditIcon } from "@shopify/polaris-icons";
import { useTrieve } from "app/context/trieveContext";

interface Policy {
  id: string;
  content: string;
}

interface PolicySettingsProps {
  shopDataset: Dataset;
  initialPolicies?: Policy[];
}

export function PolicySettings({
  shopDataset,
  initialPolicies = [],
}: PolicySettingsProps) {
  const [policies, setPolicies] = useState<Policy[]>(initialPolicies);
  const [isAdding, setIsAdding] = useState(false);
  const [editingPolicyId, setEditingPolicyId] = useState<string | null>(null);
  const [newPolicy, setNewPolicy] = useState<Policy>({
    id: "",
    content: "",
  });
  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [policyIdToDelete, setPolicyIdToDelete] = useState<string | null>(null);
  const trieve = useTrieve();

  const submit = useSubmit();

  const handlePolicyChange = async () => {
    try {
      let policyToAdd;

      if (!editingPolicyId) {
        policyToAdd = {
          ...newPolicy,
          id: String(Date.now()),
        };

        const updatedPolicies = [...policies, policyToAdd];
        setPolicies(updatedPolicies);
      } else {
        policyToAdd = {
          ...newPolicy,
          id: editingPolicyId,
        };
      }

      submit(
        {
          policy: policyToAdd.content,
          policy_id: policyToAdd.id,
          dataset_id: shopDataset.id,
          type: "policy",
        },
        {
          method: "POST",
        },
      );

      setNewPolicy({ id: "", content: "" });

      if (!editingPolicyId) {
        setIsAdding(false);
        shopify.toast.show("Policy added!");
      } else {
        setEditingPolicyId(null);
        shopify.toast.show("Policy updated!");
      }
    } catch (error) {
      shopify.toast.show("Failed to add policy. Please try again.", {
        isError: true,
      });
    }
  };

  useEffect(() => {
    trieve.trieve.getChunksGroupByTrackingId({
      groupTrackingId: "policy",
      page: 1
    }).then((res) => {
      const initialPolicies: Policy[] = res.chunks.filter((chunk) => chunk.chunk_html && chunk.tracking_id).map((chunk) => ({
        id: chunk.tracking_id ?? "",
        content: chunk.chunk_html || ""
      }))
      setPolicies(initialPolicies);
    });
  }, []);

  const handleStartEditPolicy = (policy: Policy) => {
    setIsAdding(false);
    setEditingPolicyId(policy.id);
    setNewPolicy(policy);
  };

  const handleSaveEdit = () => {
    if (!editingPolicyId) return;

    const updatedPolicies = policies.map((p) =>
      p.id === editingPolicyId ? newPolicy : p,
    );

    setPolicies(updatedPolicies);
    handlePolicyChange();
  };

  const handleDeletePolicyTrigger = (id: string) => {
    setPolicyIdToDelete(id);
    setShowDeleteConfirmModal(true);
  };

  const confirmDeletePolicy = async () => {
    if (policyIdToDelete) {
      const updatedPolicies = policies.filter((p) => p.id !== policyIdToDelete);
      setPolicies(updatedPolicies);

      await submit(
        {
          policy_id: policyIdToDelete,
          dataset_id: shopDataset.id,
          type: "delete_policy",
        },
        {
          method: "DELETE",
        },
      );

      setPolicyIdToDelete(null);
      setShowDeleteConfirmModal(false);
      shopify.toast.show("Policy deleted!");
    }
  };

  const cancelDeletePolicy = () => {
    setPolicyIdToDelete(null);
    setShowDeleteConfirmModal(false);
  };

  const handleStartAdding = () => {
    setEditingPolicyId(null);
    setIsAdding(true);
    setNewPolicy({ id: "", content: "" });
  };

  const handleCancel = () => {
    setIsAdding(false);
    setEditingPolicyId(null);
    setNewPolicy({ id: "", content: "" });
  };

  const renderItem = (item: Policy) => {
    if (editingPolicyId === item.id) {
      return (
        <Box paddingBlockEnd="400">
          <Card>
            <BlockStack gap="400">
              <TextField
                label="Policy Content"
                value={newPolicy.content}
                onChange={(value) =>
                  setNewPolicy((prev) => ({ ...prev, content: value }))
                }
                multiline={4}
                autoComplete="off"
              />
              <InlineStack gap="400">
                <Button variant="primary" onClick={handleSaveEdit}>
                  Save Changes
                </Button>
                <Button onClick={handleCancel}>Cancel</Button>
              </InlineStack>
            </BlockStack>
          </Card>
        </Box>
      );
    }

    return (
      <ResourceItem
        id={item.id}
        accessibilityLabel={`View details for policy`}
        onClick={() => handleStartEditPolicy(item)}
      >
        <InlineStack
          wrap={false}
          blockAlign="center"
          align="space-between"
          gap="200"
        >
          <Box minWidth="0" maxWidth="calc(100% - 100px)" width="100%">
            <BlockStack gap="100">
              <Text
                variant="bodyMd"
                as="p"
                fontWeight="semibold"
                truncate
                breakWord
              >
                {item.content.length > 75
                  ? `${item.content.substring(0, 75)}...`
                  : item.content}
              </Text>
            </BlockStack>
          </Box>
          <ButtonGroup>
            <Button
              icon={EditIcon}
              accessibilityLabel="Edit policy"
              onClick={() => handleStartEditPolicy(item)}
              variant="tertiary"
            />
            <Button
              icon={DeleteIcon}
              accessibilityLabel="Delete policy"
              onPointerDown={(e: React.PointerEvent) => {
                e.stopPropagation();
                handleDeletePolicyTrigger(item.id);
              }}
              variant="tertiary"
              tone="critical"
            />
          </ButtonGroup>
        </InlineStack>
      </ResourceItem>
    );
  };

  const emptyStateMarkup = (
    <EmptyState
      heading="No policies added yet"
      action={{
        content: "Add Policy",
        icon: PlusIcon,
        onAction: handleStartAdding,
      }}
      image="https://cdn.shopify.com/s/files/1/0262/4071/2726/files/emptystate-files.png"
    >
      <Text as="p" variant="bodyMd">
        Add policies to help tailor chat responses to your store's specific
        needs.
      </Text>
    </EmptyState>
  );

  return (
    <Box paddingInline="400">
      <BlockStack gap={{ xs: "800", sm: "400" }}>
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Store Policies
              </Text>
              <Text as="p" variant="bodyMd">
                Add extra information to help tailor chat responses to your store's
                specific polices.
              </Text>
            </BlockStack>
          </Box>
          <Card>
            <BlockStack gap="400">
              {policies.length > 0 ? (
                <ResourceList
                  resourceName={{ singular: "policy", plural: "policies" }}
                  items={policies}
                  renderItem={renderItem}
                />
              ) : (
                !isAdding && emptyStateMarkup
              )}

              {isAdding && (
                <Box paddingBlockStart="400" paddingBlockEnd="400">
                  <Card>
                    <BlockStack gap="400">
                      <TextField
                        label="Policy Content"
                        value={newPolicy.content}
                        onChange={(value) =>
                          setNewPolicy((prev) => ({ ...prev, content: value }))
                        }
                        multiline={4}
                        autoComplete="off"
                      />
                      <InlineStack gap="400">
                        <Button variant="primary" onClick={handlePolicyChange}>
                          Save Policy
                        </Button>
                        <Button onClick={handleCancel}>Cancel</Button>
                      </InlineStack>
                    </BlockStack>
                  </Card>
                </Box>
              )}
              {!isAdding && !editingPolicyId && policies.length > 0 && (
                <InlineStack align="end">
                  <ButtonGroup>
                    <Button
                      onClick={handleStartAdding}
                      variant="primary"
                      icon={PlusIcon}
                    >
                      Add Policy
                    </Button>
                  </ButtonGroup>
                </InlineStack>
              )}
            </BlockStack>
          </Card>
        </InlineGrid>
      </BlockStack>

      <Modal
        open={showDeleteConfirmModal}
        onClose={cancelDeletePolicy}
        title="Confirm Deletion"
        primaryAction={{
          content: "Delete",
          onAction: confirmDeletePolicy,
          destructive: true,
        }}
        secondaryActions={[
          {
            content: "Cancel",
            onAction: cancelDeletePolicy,
          },
        ]}
      >
        <Modal.Section>
          <BlockStack>
            <Text as="p">Are you sure you want to delete this policy?</Text>
          </BlockStack>
        </Modal.Section>
      </Modal>
    </Box>
  );
}
