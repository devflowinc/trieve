# OpenGraphMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** |  | [optional] 
**image** | **str** |  | [optional] 
**title** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.open_graph_metadata import OpenGraphMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of OpenGraphMetadata from a JSON string
open_graph_metadata_instance = OpenGraphMetadata.from_json(json)
# print the JSON string representation of the object
print(OpenGraphMetadata.to_json())

# convert the object into a dict
open_graph_metadata_dict = open_graph_metadata_instance.to_dict()
# create an instance of OpenGraphMetadata from a dict
open_graph_metadata_form_dict = open_graph_metadata.from_dict(open_graph_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


