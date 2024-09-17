# RAGUsageGraph


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RAGAnalyticsFilter**](RAGAnalyticsFilter.md) |  | [optional] 
**granularity** | [**Granularity**](Granularity.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.rag_usage_graph import RAGUsageGraph

# TODO update the JSON string below
json = "{}"
# create an instance of RAGUsageGraph from a JSON string
rag_usage_graph_instance = RAGUsageGraph.from_json(json)
# print the JSON string representation of the object
print(RAGUsageGraph.to_json())

# convert the object into a dict
rag_usage_graph_dict = rag_usage_graph_instance.to_dict()
# create an instance of RAGUsageGraph from a dict
rag_usage_graph_form_dict = rag_usage_graph.from_dict(rag_usage_graph_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


