# LatencyGraphResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**latency_points** | [**List[SearchLatencyGraph]**](SearchLatencyGraph.md) |  | 

## Example

```python
from trieve_py_client.models.latency_graph_response import LatencyGraphResponse

# TODO update the JSON string below
json = "{}"
# create an instance of LatencyGraphResponse from a JSON string
latency_graph_response_instance = LatencyGraphResponse.from_json(json)
# print the JSON string representation of the object
print(LatencyGraphResponse.to_json())

# convert the object into a dict
latency_graph_response_dict = latency_graph_response_instance.to_dict()
# create an instance of LatencyGraphResponse from a dict
latency_graph_response_form_dict = latency_graph_response.from_dict(latency_graph_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


