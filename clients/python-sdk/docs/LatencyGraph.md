# LatencyGraph


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**granularity** | [**Granularity**](Granularity.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.latency_graph import LatencyGraph

# TODO update the JSON string below
json = "{}"
# create an instance of LatencyGraph from a JSON string
latency_graph_instance = LatencyGraph.from_json(json)
# print the JSON string representation of the object
print(LatencyGraph.to_json())

# convert the object into a dict
latency_graph_dict = latency_graph_instance.to_dict()
# create an instance of LatencyGraph from a dict
latency_graph_form_dict = latency_graph.from_dict(latency_graph_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


