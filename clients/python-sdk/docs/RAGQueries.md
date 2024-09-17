# RAGQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RAGAnalyticsFilter**](RAGAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**sort_by** | [**RAGSortBy**](RAGSortBy.md) |  | [optional] 
**sort_order** | [**SortOrder**](SortOrder.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.rag_queries import RAGQueries

# TODO update the JSON string below
json = "{}"
# create an instance of RAGQueries from a JSON string
rag_queries_instance = RAGQueries.from_json(json)
# print the JSON string representation of the object
print(RAGQueries.to_json())

# convert the object into a dict
rag_queries_dict = rag_queries_instance.to_dict()
# create an instance of RAGQueries from a dict
rag_queries_form_dict = rag_queries.from_dict(rag_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


