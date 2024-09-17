# EventTypesOneOf2


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clicked_items** | [**ChunksWithPositions**](ChunksWithPositions.md) |  | 
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**request_id** | **str** | The request id of the event to associate it with a request | [optional] 
**user_id** | **str** | The user id of the user who clicked the items | [optional] 

## Example

```python
from trieve_py_client.models.event_types_one_of2 import EventTypesOneOf2

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypesOneOf2 from a JSON string
event_types_one_of2_instance = EventTypesOneOf2.from_json(json)
# print the JSON string representation of the object
print(EventTypesOneOf2.to_json())

# convert the object into a dict
event_types_one_of2_dict = event_types_one_of2_instance.to_dict()
# create an instance of EventTypesOneOf2 from a dict
event_types_one_of2_form_dict = event_types_one_of2.from_dict(event_types_one_of2_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


