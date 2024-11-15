# GetEventsResponseBody

Response body for the GetEvents endpoint

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**events** | [**List[EventData]**](EventData.md) |  | 

## Example

```python
from trieve_py_client.models.get_events_response_body import GetEventsResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of GetEventsResponseBody from a JSON string
get_events_response_body_instance = GetEventsResponseBody.from_json(json)
# print the JSON string representation of the object
print(GetEventsResponseBody.to_json())

# convert the object into a dict
get_events_response_body_dict = get_events_response_body_instance.to_dict()
# create an instance of GetEventsResponseBody from a dict
get_events_response_body_form_dict = get_events_response_body.from_dict(get_events_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


