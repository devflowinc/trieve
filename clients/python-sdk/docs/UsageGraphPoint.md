# UsageGraphPoint


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**requests** | **int** |  | 
**time_stamp** | **str** |  | 

## Example

```python
from trieve_py_client.models.usage_graph_point import UsageGraphPoint

# TODO update the JSON string below
json = "{}"
# create an instance of UsageGraphPoint from a JSON string
usage_graph_point_instance = UsageGraphPoint.from_json(json)
# print the JSON string representation of the object
print(UsageGraphPoint.to_json())

# convert the object into a dict
usage_graph_point_dict = usage_graph_point_instance.to_dict()
# create an instance of UsageGraphPoint from a dict
usage_graph_point_form_dict = usage_graph_point.from_dict(usage_graph_point_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


