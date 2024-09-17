# SearchAnalyticsFilter


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 
**search_method** | [**SearchMethod**](SearchMethod.md) |  | [optional] 
**search_type** | [**SearchType**](SearchType.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.search_analytics_filter import SearchAnalyticsFilter

# TODO update the JSON string below
json = "{}"
# create an instance of SearchAnalyticsFilter from a JSON string
search_analytics_filter_instance = SearchAnalyticsFilter.from_json(json)
# print the JSON string representation of the object
print(SearchAnalyticsFilter.to_json())

# convert the object into a dict
search_analytics_filter_dict = search_analytics_filter_instance.to_dict()
# create an instance of SearchAnalyticsFilter from a dict
search_analytics_filter_form_dict = search_analytics_filter.from_dict(search_analytics_filter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


