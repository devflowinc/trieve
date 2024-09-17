# RecommendationStrategy

Strategy to use for recommendations, either \"average_vector\" or \"best_score\". The default is \"average_vector\". The \"average_vector\" strategy will construct a single average vector from the positive and negative samples then use it to perform a pseudo-search. The \"best_score\" strategy is more advanced and navigates the HNSW with a heuristic of picking edges where the point is closer to the positive samples than it is the negatives.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


