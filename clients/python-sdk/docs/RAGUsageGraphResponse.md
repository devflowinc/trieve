# RAGUsageGraphResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**usage_points** | [**List[UsageGraphPoint]**](UsageGraphPoint.md) |  | 

## Example

```python
from trieve_py_client.models.rag_usage_graph_response import RAGUsageGraphResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RAGUsageGraphResponse from a JSON string
rag_usage_graph_response_instance = RAGUsageGraphResponse.from_json(json)
# print the JSON string representation of the object
print(RAGUsageGraphResponse.to_json())

# convert the object into a dict
rag_usage_graph_response_dict = rag_usage_graph_response_instance.to_dict()
# create an instance of RAGUsageGraphResponse from a dict
rag_usage_graph_response_form_dict = rag_usage_graph_response.from_dict(rag_usage_graph_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


