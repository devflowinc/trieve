import {
  BlockStack,
  Text,
  Card,
  TextField,
  InlineStack,
  Button,
} from "@shopify/polaris";
import { useEffect, useState } from "react";
import { Dataset } from "trieve-ts-sdk";
import { useSubmit } from "@remix-run/react";
import { useTrieve } from "app/context/trieveContext";
import { BuilderView, EditFormProps } from "../BuilderView";

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
  const trieve = useTrieve();
  const submit = useSubmit();

  useEffect(() => {
    trieve.trieve
      .getChunksGroupByTrackingId({
        groupTrackingId: "policy",
        page: 1,
      })
      .then((res) => {
        const loadedPolicies: Policy[] = res.chunks
          .filter((chunk) => chunk.chunk_html && chunk.tracking_id)
          .map((chunk) => ({
            id: chunk.tracking_id ?? "",
            content: chunk.chunk_html || "",
          }));
        setPolicies(loadedPolicies);
      });
  }, []);

  const handlePoliciesChange = (updatedPolicies: Policy[]) => {
    setPolicies(updatedPolicies);
  };

  const handleSaveSuccess = async (policy: Policy, isNew: boolean) => {
    submit(
      {
        policy: policy.content,
        policy_id: policy.id,
        dataset_id: shopDataset.id,
        type: "policy",
      },
      {
        method: "POST",
      },
    );

    shopify.toast.show(isNew ? "Policy added!" : "Policy updated!");
  };

  const handleDeletePolicy = async (policyId: string) => {
    await submit(
      {
        policy_id: policyId,
        dataset_id: shopDataset.id,
        type: "delete_policy",
      },
      {
        method: "DELETE",
      },
    );

    shopify.toast.show("Policy deleted!");
  };

  const renderPolicyContent = (policy: Policy) => (
    <BlockStack gap="100">
      <Text variant="bodyMd" as="p" fontWeight="semibold" truncate breakWord>
        {policy.content.length > 75
          ? `${policy.content.substring(0, 75)}...`
          : policy.content}
      </Text>
    </BlockStack>
  );

  const renderPolicyEditForm = ({
    item,
    onChange,
    onSave,
    onCancel,
  }: EditFormProps<Policy>) => (
    <Card>
      <BlockStack gap="400">
        <TextField
          label="Policy Content"
          value={item.content}
          onChange={(value) => onChange({ ...item, content: value })}
          multiline={4}
          autoComplete="off"
        />
        <InlineStack gap="400">
          <Button variant="primary" onClick={onSave}>
            Save {item.id ? "Changes" : "Policy"}
          </Button>
          <Button onClick={onCancel}>Cancel</Button>
        </InlineStack>
      </BlockStack>
    </Card>
  );

  const validatePolicy = (policy: Policy) => {
    if (!policy.content.trim()) {
      shopify.toast.show("Policy content cannot be empty", { isError: true });
      return false;
    }
    return true;
  };

  return (
    <BuilderView
      items={policies}
      onItemsChange={(updatedPolicies) => {
        setPolicies(updatedPolicies);
        const deletedPolicyId = policies.find(
          (p) => !updatedPolicies.some((up) => up.id === p.id),
        )?.id;
        if (deletedPolicyId) {
          handleDeletePolicy(deletedPolicyId);
        }
      }}
      renderItemContent={renderPolicyContent}
      renderEditForm={renderPolicyEditForm}
      labels={{
        singular: "policy",
        plural: "policies",
        addButton: "Add Policy",
        editTitle: "Edit Policy",
        addTitle: "Add New Policy",
        emptyStateHeading: "No policies added yet",
        emptyStateDescription:
          "Add policies to help tailor chat responses to your store's specific needs.",
        deleteConfirmMessage: "Are you sure you want to delete this policy?",
      }}
      validateItem={validatePolicy}
      onSaveSuccess={handleSaveSuccess}
      header={{
        title: "Store Policies",
        subtitle:
          "Add extra information to help tailor chat responses to your store's specific polices.",
      }}
    />
  );
}
