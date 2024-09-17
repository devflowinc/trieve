# SortBySearchType


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**prefetch_amount** | **int** | How many results to pull in before the rerabj | [optional] 
**rerank_query** | **str** | Query to use for prefetching defaults to the search query | [optional] 
**rerank_type** | [**ReRankOptions**](ReRankOptions.md) |  | 

## Example

```python
from trieve_py_client.models.sort_by_search_type import SortBySearchType

# TODO update the JSON string below
json = "{}"
# create an instance of SortBySearchType from a JSON string
sort_by_search_type_instance = SortBySearchType.from_json(json)
# print the JSON string representation of the object
print(SortBySearchType.to_json())

# convert the object into a dict
sort_by_search_type_dict = sort_by_search_type_instance.to_dict()
# create an instance of SortBySearchType from a dict
sort_by_search_type_form_dict = sort_by_search_type.from_dict(sort_by_search_type_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


