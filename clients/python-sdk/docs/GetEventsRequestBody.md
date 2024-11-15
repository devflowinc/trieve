# GetEventsRequestBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**EventAnalyticsFilter**](EventAnalyticsFilter.md) |  | [optional] 
**page** | **int** | Page of results to return | [optional] 

## Example

```python
from trieve_py_client.models.get_events_request_body import GetEventsRequestBody

# TODO update the JSON string below
json = "{}"
# create an instance of GetEventsRequestBody from a JSON string
get_events_request_body_instance = GetEventsRequestBody.from_json(json)
# print the JSON string representation of the object
print(GetEventsRequestBody.to_json())

# convert the object into a dict
get_events_request_body_dict = get_events_request_body_instance.to_dict()
# create an instance of GetEventsRequestBody from a dict
get_events_request_body_form_dict = get_events_request_body.from_dict(get_events_request_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


