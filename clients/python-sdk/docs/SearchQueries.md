# SearchQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**sort_by** | [**SearchSortBy**](SearchSortBy.md) |  | [optional] 
**sort_order** | [**SortOrder**](SortOrder.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_queries import SearchQueries

# TODO update the JSON string below
json = "{}"
# create an instance of SearchQueries from a JSON string
search_queries_instance = SearchQueries.from_json(json)
# print the JSON string representation of the object
print(SearchQueries.to_json())

# convert the object into a dict
search_queries_dict = search_queries_instance.to_dict()
# create an instance of SearchQueries from a dict
search_queries_form_dict = search_queries.from_dict(search_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


