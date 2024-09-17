# QdrantSortBy

Sort by lets you specify a method to sort the results by. If not specified, this defaults to the score of the chunks. If specified, this can be any key in the chunk metadata. This key must be a numeric value within the payload.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**direction** | [**SortOrder**](SortOrder.md) |  | [optional] 
**field** | **str** | Field to sort by. This has to be a numeric field with a Qdrant &#x60;Range&#x60; index on it. i.e. num_value and timestamp | 
**prefetch_amount** | **int** | How many results to pull in before the rerabj | [optional] 
**rerank_query** | **str** | Query to use for prefetching defaults to the search query | [optional] 
**rerank_type** | [**ReRankOptions**](ReRankOptions.md) |  | 

## Example

```python
from trieve_py_client.models.qdrant_sort_by import QdrantSortBy

# TODO update the JSON string below
json = "{}"
# create an instance of QdrantSortBy from a JSON string
qdrant_sort_by_instance = QdrantSortBy.from_json(json)
# print the JSON string representation of the object
print(QdrantSortBy.to_json())

# convert the object into a dict
qdrant_sort_by_dict = qdrant_sort_by_instance.to_dict()
# create an instance of QdrantSortBy from a dict
qdrant_sort_by_form_dict = qdrant_sort_by.from_dict(qdrant_sort_by_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


