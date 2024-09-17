# GetEventsData


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_types** | [**List[EventTypeRequest]**](EventTypeRequest.md) | The types of events to get. Any combination of file_uploaded, chunk_uploaded, chunk_action_failed, chunk_updated, or qdrant_index_failed. Leave undefined to get all events. | [optional] 
**page** | **int** | The page number to get. Default is 1. | [optional] 
**page_size** | **int** | The number of items per page. Default is 10. | [optional] 

## Example

```python
from trieve_py_client.models.get_events_data import GetEventsData

# TODO update the JSON string below
json = "{}"
# create an instance of GetEventsData from a JSON string
get_events_data_instance = GetEventsData.from_json(json)
# print the JSON string representation of the object
print(GetEventsData.to_json())

# convert the object into a dict
get_events_data_dict = get_events_data_instance.to_dict()
# create an instance of GetEventsData from a dict
get_events_data_form_dict = get_events_data.from_dict(get_events_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


