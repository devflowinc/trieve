# EventTypesOneOf


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**items** | **List[str]** | The items that were viewed | 
**metadata** | **object** | Any other metadata associated with the event | [optional] 
**request_id** | **str** | The request id of the event to associate it with a request | [optional] 
**user_id** | **str** | The user id of the user who viewed the items | [optional] 

## Example

```python
from trieve_py_client.models.event_types_one_of import EventTypesOneOf

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypesOneOf from a JSON string
event_types_one_of_instance = EventTypesOneOf.from_json(json)
# print the JSON string representation of the object
print(EventTypesOneOf.to_json())

# convert the object into a dict
event_types_one_of_dict = event_types_one_of_instance.to_dict()
# create an instance of EventTypesOneOf from a dict
event_types_one_of_form_dict = event_types_one_of.from_dict(event_types_one_of_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


