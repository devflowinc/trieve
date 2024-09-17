# EventTypesOneOf4


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**items** | **Dict[str, str]** | The filter items that were clicked in a hashmap ie. {filter_name: filter_value} where filter_name is filter_type::field_name | 
**request_id** | **str** | The request id of the event to associate it with a request | [optional] 
**user_id** | **str** | The user id of the user who clicked the items | [optional] 

## Example

```python
from trieve_py_client.models.event_types_one_of4 import EventTypesOneOf4

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypesOneOf4 from a JSON string
event_types_one_of4_instance = EventTypesOneOf4.from_json(json)
# print the JSON string representation of the object
print(EventTypesOneOf4.to_json())

# convert the object into a dict
event_types_one_of4_dict = event_types_one_of4_instance.to_dict()
# create an instance of EventTypesOneOf4 from a dict
event_types_one_of4_form_dict = event_types_one_of4.from_dict(event_types_one_of4_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


