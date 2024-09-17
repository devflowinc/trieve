# SearchUsageGraphResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**usage_points** | [**List[UsageGraphPoint]**](UsageGraphPoint.md) |  | 

## Example

```python
from trieve_py_client.models.search_usage_graph_response import SearchUsageGraphResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SearchUsageGraphResponse from a JSON string
search_usage_graph_response_instance = SearchUsageGraphResponse.from_json(json)
# print the JSON string representation of the object
print(SearchUsageGraphResponse.to_json())

# convert the object into a dict
search_usage_graph_response_dict = search_usage_graph_response_instance.to_dict()
# create an instance of SearchUsageGraphResponse from a dict
search_usage_graph_response_form_dict = search_usage_graph_response.from_dict(search_usage_graph_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


