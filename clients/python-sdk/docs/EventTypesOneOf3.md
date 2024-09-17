# EventTypesOneOf3


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**currency** | **str** | The currency of the purchase | [optional] 
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**items** | **List[str]** | The items that were purchased | 
**request_id** | **str** | The request id of the event to associate it with a request | [optional] 
**user_id** | **str** | The user id of the user who purchased the items | [optional] 
**value** | **float** | The value of the purchase | [optional] 

## Example

```python
from trieve_py_client.models.event_types_one_of3 import EventTypesOneOf3

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypesOneOf3 from a JSON string
event_types_one_of3_instance = EventTypesOneOf3.from_json(json)
# print the JSON string representation of the object
print(EventTypesOneOf3.to_json())

# convert the object into a dict
event_types_one_of3_dict = event_types_one_of3_instance.to_dict()
# create an instance of EventTypesOneOf3 from a dict
event_types_one_of3_form_dict = event_types_one_of3.from_dict(event_types_one_of3_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


