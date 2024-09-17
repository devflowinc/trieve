# SearchUsageGraph


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**granularity** | [**Granularity**](Granularity.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_usage_graph import SearchUsageGraph

# TODO update the JSON string below
json = "{}"
# create an instance of SearchUsageGraph from a JSON string
search_usage_graph_instance = SearchUsageGraph.from_json(json)
# print the JSON string representation of the object
print(SearchUsageGraph.to_json())

# convert the object into a dict
search_usage_graph_dict = search_usage_graph_instance.to_dict()
# create an instance of SearchUsageGraph from a dict
search_usage_graph_form_dict = search_usage_graph.from_dict(search_usage_graph_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


