import React, { useState, useCallback, useEffect } from "react";
import { Text, BlockStack } from "@shopify/polaris";
import { useSubmit } from "@remix-run/react";
import { QuestionEditBlock } from "../QuestionEditBlock";
import { useTrieve } from "app/context/trieveContext";
import { ChunkMetadata } from "trieve-ts-sdk";
import { BuilderView, EditFormProps } from "../BuilderView";

export interface PresetQuestion {
  id: string;
  questionText: string;
  promptForAI?: string;
  products?: Product[];
}

export interface PresetQuestionsProps {
  initialQuestions: PresetQuestion[];
}

export interface Product {
  id: string;
  image: string;
  price: number;
  url: string;
  title: string;
  groupId: string;
}

export function PresetQuestions({ initialQuestions }: PresetQuestionsProps) {
  const submit = useSubmit();
  const trieve = useTrieve();

  const [presetQuestions, setPresetQuestions] = useState<PresetQuestion[]>(
    initialQuestions || [],
  );
  const [searchedProducts, setSearchedProducts] = useState<Product[]>([]);
  const [currentSearchTerm, setCurrentSearchTerm] = useState<string>("");

  const submitPresetQuestions = useCallback(
    (questions: PresetQuestion[]) => {
      submit(
        {
          type: "preset-questions",
          presetQuestions: JSON.stringify(questions),
        },
        {
          method: "POST",
        },
      );
    },
    [submit],
  );

  const handleQuestionsChange = (updatedQuestions: PresetQuestion[]) => {
    setPresetQuestions(updatedQuestions);
    submitPresetQuestions(updatedQuestions);
  };

  const handleSaveSuccess = (question: PresetQuestion, isNew: boolean) => {
    shopify.toast.show("Preset question saved!");
  };

  useEffect(() => {
    const getProducts = async (searchTerm: string) => {
      if (!trieve.trieve || searchTerm.trim().length === 0) {
        setSearchedProducts([]);
        return;
      }
      try {
        const results = await trieve.trieve.searchOverGroups({
          query: searchTerm,
          search_type: "fulltext",
        });

        const apiResults: Product[] = results.results
          .map((result) => {
            const chunk = result.chunks[0]?.chunk as ChunkMetadata | undefined;
            return {
              id: chunk?.id || String(Date.now() + Math.random()),
              image: chunk?.image_urls?.[0] || "",
              price: chunk?.num_value || 0,
              url: chunk?.link || "",
              title: (chunk?.metadata as any)?.title || "Untitled Product",
              groupId: result.group.id || "",
            };
          })
          .filter((p) => p.id && p.title !== "Untitled Product");
        setSearchedProducts(apiResults);
      } catch (error) {
        console.error("Failed to fetch products:", error);
        setSearchedProducts([]);
      }
    };

    if (!currentSearchTerm || currentSearchTerm.trim().length === 0) {
      setSearchedProducts([]);
      return;
    }

    const handler = setTimeout(() => {
      if (currentSearchTerm) {
        getProducts(currentSearchTerm);
      }
    }, 100);

    return () => {
      clearTimeout(handler);
    };
  }, [currentSearchTerm, trieve.trieve]);

  const renderQuestionContent = (question: PresetQuestion) => (
    <BlockStack gap="100">
      <Text variant="bodyMd" as="p" fontWeight="semibold" truncate>
        {question.questionText}
      </Text>
      <Text variant="bodySm" as="p" tone="subdued">
        Targets {question.products?.length ?? 0} product(s)
      </Text>
    </BlockStack>
  );

  const renderQuestionEditForm = ({
    item,
    onChange,
    onSave,
    onCancel,
    isNew,
  }: EditFormProps<PresetQuestion>) => {
    const handleQuestionSelectedProductsChange = (
      updatedSelectedProducts: Product[],
    ) => {
      onChange({ ...item, products: updatedSelectedProducts });
    };

    const handleRemoveSelectedProduct = (productIdToRemove: string) => {
      onChange({
        ...item,
        products:
          item.products?.filter((p) => p.id !== productIdToRemove) || [],
      });
    };

    return (
      <QuestionEditBlock
        title={isNew ? "Add New Preset Question" : "Edit Preset Question"}
        questionText={item.questionText}
        onQuestionTextChange={(value: string) =>
          onChange({ ...item, questionText: value })
        }
        productSearchTerm={currentSearchTerm}
        onProductSearchTermChange={setCurrentSearchTerm}
        promptForAI={item.promptForAI || ""}
        onPromptForAIChange={(value: string) =>
          onChange({
            ...item,
            promptForAI: value.length > 0 ? value : undefined,
          })
        }
        products={searchedProducts}
        selectedProducts={item.products || []}
        onSelectedProductsChange={handleQuestionSelectedProductsChange}
        onRemoveSelectedProduct={handleRemoveSelectedProduct}
        onSave={onSave}
        onCancel={() => {
          setCurrentSearchTerm("");
          setSearchedProducts([]);
          onCancel();
        }}
      />
    );
  };

  const validateQuestion = (question: PresetQuestion) => {
    if (!question.questionText.trim()) {
      shopify.toast.show("Question text cannot be empty", { isError: true });
      return false;
    }
    return true;
  };

  return (
    <BuilderView
      items={presetQuestions}
      onItemsChange={handleQuestionsChange}
      renderItemContent={renderQuestionContent}
      renderEditForm={renderQuestionEditForm}
      labels={{
        singular: "question",
        plural: "questions",
        addButton: "Add Suggested Question",
        editTitle: "Edit Preset Question",
        addTitle: "Add New Preset Question",
        emptyStateHeading: "Define suggested questions for your customers",
        emptyStateDescription:
          "Create preset questions to guide users and help them quickly find information about specific products.",
        deleteConfirmMessage:
          "Are you sure you want to delete this preset question?",
      }}
      validateItem={validateQuestion}
      onSaveSuccess={handleSaveSuccess}
      header={{
        title: "Global Preset Questions",
        subtitle:
          "These questions will be displayed in the global chat UI as preset questions.",
      }}
    />
  );
}
