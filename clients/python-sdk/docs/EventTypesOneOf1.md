# EventTypesOneOf1


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**items** | **List[str]** | The items that were added to the cart | 
**metadata** | **object** | Any other metadata associated with the event | [optional] 
**request_id** | **str** | The request id of the event to associate it with a request | [optional] 
**user_id** | **str** | The user id of the user who added the items to the cart | [optional] 

## Example

```python
from trieve_py_client.models.event_types_one_of1 import EventTypesOneOf1

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypesOneOf1 from a JSON string
event_types_one_of1_instance = EventTypesOneOf1.from_json(json)
# print the JSON string representation of the object
print(EventTypesOneOf1.to_json())

# convert the object into a dict
event_types_one_of1_dict = event_types_one_of1_instance.to_dict()
# create an instance of EventTypesOneOf1 from a dict
event_types_one_of1_form_dict = event_types_one_of1.from_dict(event_types_one_of1_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


