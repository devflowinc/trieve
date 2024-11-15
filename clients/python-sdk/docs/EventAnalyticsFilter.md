# EventAnalyticsFilter

Filter to apply to the events when querying for them

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 
**event_type** | [**EventTypesFilter**](EventTypesFilter.md) |  | [optional] 
**is_conversion** | **bool** | Filter by conversions | [optional] 
**metadata_filter** | **str** | Filter by metadata path i.e. path.attribute &#x3D; \\\&quot;value\\\&quot; | [optional] 
**user_id** | **str** | Filter by user ID | [optional] 

## Example

```python
from trieve_py_client.models.event_analytics_filter import EventAnalyticsFilter

# TODO update the JSON string below
json = "{}"
# create an instance of EventAnalyticsFilter from a JSON string
event_analytics_filter_instance = EventAnalyticsFilter.from_json(json)
# print the JSON string representation of the object
print(EventAnalyticsFilter.to_json())

# convert the object into a dict
event_analytics_filter_dict = event_analytics_filter_instance.to_dict()
# create an instance of EventAnalyticsFilter from a dict
event_analytics_filter_form_dict = event_analytics_filter.from_dict(event_analytics_filter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


