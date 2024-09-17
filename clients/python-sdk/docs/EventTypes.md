# EventTypes


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**items** | **Dict[str, str]** | The filter items that were clicked in a hashmap ie. {filter_name: filter_value} where filter_name is filter_type::field_name | 
**metadata** | **object** | Any other metadata associated with the event | [optional] 
**request_id** | **str** | The request id of the event to associate it with a request | [optional] 
**user_id** | **str** | The user id of the user who clicked the items | [optional] 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**clicked_items** | [**ChunksWithPositions**](ChunksWithPositions.md) |  | 
**currency** | **str** | The currency of the purchase | [optional] 
**value** | **float** | The value of the purchase | [optional] 

## Example

```python
from trieve_py_client.models.event_types import EventTypes

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypes from a JSON string
event_types_instance = EventTypes.from_json(json)
# print the JSON string representation of the object
print(EventTypes.to_json())

# convert the object into a dict
event_types_dict = event_types_instance.to_dict()
# create an instance of EventTypes from a dict
event_types_form_dict = event_types.from_dict(event_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


