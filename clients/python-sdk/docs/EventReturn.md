# EventReturn


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**events** | [**List[WorkerEvent]**](WorkerEvent.md) |  | 
**page_count** | **int** |  | 

## Example

```python
from trieve_py_client.models.event_return import EventReturn

# TODO update the JSON string below
json = "{}"
# create an instance of EventReturn from a JSON string
event_return_instance = EventReturn.from_json(json)
# print the JSON string representation of the object
print(EventReturn.to_json())

# convert the object into a dict
event_return_dict = event_return_instance.to_dict()
# create an instance of EventReturn from a dict
event_return_form_dict = event_return.from_dict(event_return_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


