# SortOptions

Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**location_bias** | [**GeoInfoWithBias**](GeoInfoWithBias.md) |  | [optional] 
**mmr** | [**MmrOptions**](MmrOptions.md) |  | [optional] 
**recency_bias** | **float** | Recency Bias lets you determine how much of an effect the recency of chunks will have on the search results. If not specified, this defaults to 0.0. We recommend setting this to 1.0 for a gentle reranking of the results, &gt;3.0 for a strong reranking of the results. | [optional] 
**sort_by** | [**QdrantSortBy**](QdrantSortBy.md) |  | [optional] 
**tag_weights** | **Dict[str, float]** | Tag weights is a JSON object which can be used to boost the ranking of chunks with certain tags. This is useful for when you want to be able to bias towards chunks with a certain tag on the fly. The keys are the tag names and the values are the weights. | [optional] 
**use_weights** | **bool** | Set use_weights to true to use the weights of the chunks in the result set in order to sort them. If not specified, this defaults to true. | [optional] 

## Example

```python
from trieve_py_client.models.sort_options import SortOptions

# TODO update the JSON string below
json = "{}"
# create an instance of SortOptions from a JSON string
sort_options_instance = SortOptions.from_json(json)
# print the JSON string representation of the object
print(SortOptions.to_json())

# convert the object into a dict
sort_options_dict = sort_options_instance.to_dict()
# create an instance of SortOptions from a dict
sort_options_form_dict = sort_options.from_dict(sort_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


