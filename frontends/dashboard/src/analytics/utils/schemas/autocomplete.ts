import { PublicPageParameters, SearchChunksReqPayload } from "trieve-ts-sdk";
import { z } from "zod";

export const searchMethodEnum = z.enum([
  "fulltext",
  "semantic",
  "hybrid",
  "bm25",
]);

const typoRangeSchema = z
  .object({
    // Define TypoRange schema here if needed
    max: z.number().nullish(),
    min: z.number(),
  })
  .nullish();

export const typoOptionsSchema = z.object({
  correct_typos: z.boolean().nullish(),
  disable_on_word: z.array(z.string()).nullish(),
  one_typo_word_range: typoRangeSchema,
  prioritize_domain_specifc_words: z.boolean().nullish(),
  two_typo_word_range: typoRangeSchema,
});

const conditionTypeSchema = z.any(); // Define proper schema if needed

export const chunkFilterSchema = z.object({
  jsonb_prefilter: z.boolean().nullish(),
  must: z.array(conditionTypeSchema).nullish(),
  must_not: z.array(conditionTypeSchema).nullish(),
  should: z.array(conditionTypeSchema).nullish(),
});

const fullTextBoostSchema = z.object({
  boost_factor: z.number(),
  phrase: z.string(),
});

const semanticBoostSchema = z.object({
  distance_factor: z.number(),
  phrase: z.string(),
});

const scoringOptionsSchema = z.object({
  fulltext_boost: fullTextBoostSchema.nullish(),
  semantic_boost: semanticBoostSchema.nullish(),
});

const geoInfoWithBiasSchema = z.any(); // Define proper schema if needed
const qdrantSortBySchema = z.any(); // Define proper schema if needed

const sortOptionsSchema = z.object({
  location_bias: geoInfoWithBiasSchema.nullable(),
  sort_by: qdrantSortBySchema.nullable(),
  tag_weights: z.record(z.string(), z.number()).nullish(),
  use_weights: z.boolean().nullish(),
});

export const publicPageSearchOptionsSchema = z
  .object({
    content_only: z.boolean().nullish(),
    filters: chunkFilterSchema.nullish(),
    get_total_pages: z.boolean().nullish(),
    page: z.number().nullish(),
    page_size: z.number().nullish(),
    remove_stop_words: z.boolean().nullish(),
    score_threshold: z.number().nullish(),
    scoring_options: scoringOptionsSchema.nullish(),
    search_type: searchMethodEnum.nullish(),
    slim_chunks: z.boolean().nullish(),
    sort_options: sortOptionsSchema.nullish(),
    typo_options: typoOptionsSchema.nullish(),
    use_quote_negated_terms: z.boolean().nullish(),
    use_autocomplete: z.boolean().nullish(),
    user_id: z.string().nullish(),
  })
  .strict();

export const tagPropSchema = z.array(
  z.object({
    tag: z.string(),
    label: z.string().nullish(),
    selected: z.boolean().nullish(),
    iconClassName: z.string().nullish(),
    icon: z.function().args().returns(z.void()).nullish(),
    description: z.string().nullish(),
  }),
);

// Type test to stay consistent
type LibraryOptions = Omit<
  SearchChunksReqPayload,
  "query" | "highlight_options" | "search_type"
> & {
  use_autocomplete?: boolean | null;
};

// Assert that demo page search options are valid at type level
type AssertValidOptions = z.infer<typeof publicPageSearchOptionsSchema>;
// This will error if the modal prop types get out of sync with the schema
export const assertValidOptions = (
  options: LibraryOptions,
): asserts options is AssertValidOptions => {
  publicPageSearchOptionsSchema.parse(options);
};

// This will error if the modal prop types get out of sync with the schema
export const assertValidOptions2 = (
  options: LibraryOptions,
): asserts options is NonNullable<PublicPageParameters["searchOptions"]> => {
  publicPageSearchOptionsSchema.parse(options);
};
